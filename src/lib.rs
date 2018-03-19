extern crate tritium;//C:/Program Files (x86)/Microsoft Games/Halo/MAPS/bloodgulch.map
use tritium::map::*;
use tritium::resource_map::*;

extern crate byteorder;
use byteorder::{ByteOrder,LittleEndian};

#[no_mangle]
pub unsafe extern "C" fn convert_map_cd(
    buffer : *mut u8,               buffer_len : usize,
    map_data_raw : *const u8,       map_data_len : usize,
    multiplayer_raw : *const u8,    multiplayer_len : usize,
    bitmaps_pc_raw : *const u8,     bitmaps_pc_len : usize,
    bitmaps_ce_raw : *const u8,     bitmaps_ce_len : usize,
    sounds_pc_raw : *const u8,      sounds_pc_len : usize,
    sounds_ce_raw : *const u8,      sounds_ce_len : usize
) -> usize {
    use std::slice::{from_raw_parts, from_raw_parts_mut};
    let map_data = from_raw_parts(map_data_raw, map_data_len);
    let multiplayer = from_raw_parts(multiplayer_raw, multiplayer_len);
    let bitmaps_pc = from_raw_parts(bitmaps_pc_raw, bitmaps_pc_len);
    let sounds_pc = from_raw_parts(sounds_pc_raw, sounds_pc_len);
    let bitmaps_ce = if bitmaps_ce_len == 0 {
        None
    }
    else {
        Some(from_raw_parts(bitmaps_ce_raw, bitmaps_ce_len))
    };
    let sounds_ce = if sounds_ce_len == 0 {
        None
    }
    else {
        Some(from_raw_parts(sounds_ce_raw, sounds_ce_len))
    };

    // Debugging
    println!("Received map_data_len of {}...", map_data_len);
    println!("Received multiplayer_len of {}...", multiplayer_len);
    println!("Received bitmaps_pc_len of {}...", bitmaps_pc_len);
    println!("Received bitmaps_ce_len of {}...", bitmaps_ce_len);
    println!("Received sounds_pc_len of {}...", sounds_pc_len);
    println!("Received sounds_ce_len of {}...", sounds_ce_len);
    println!("about to convert map...");

    match convert_map(map_data,multiplayer,bitmaps_pc,sounds_pc,bitmaps_ce,sounds_ce) {
        Ok(n) => {
            let l = n.len();
            if l > buffer_len {
                println!("Success! {}", l);
                0
            }
            else {
                println!("Warning: return value not greater than buffer length...");
                let b = from_raw_parts_mut(buffer, buffer_len);
                for i in 0..n.len() {
                    b[i] = n[i];
                }
                l
            }
        },
        Err(_) => {
            println!("Error!");
            0
        }
    }
}

fn convert_map(map_data : &[u8], multiplayer : &[u8], bitmaps_pc : &[u8], sounds_pc : &[u8], bitmaps_ce : Option<&[u8]>, sounds_ce : Option<&[u8]>) -> Result<Vec<u8>,&'static str> {
    let mut map = try!(Map::from_cache_file(&map_data));

    // @todo - temporary until I know what needs to be passed for multiplayer...
    if multiplayer.len() > 0 {
        let multiplayer = try!(Map::from_cache_file(&multiplayer));
        let scenario_type = &multiplayer.tag_array.tags()[multiplayer.tag_array.find_tag("ui\\ui_tags_loaded_multiplayer_scenario_type",0x74616763).unwrap()];
        for i in scenario_type.references(&multiplayer.tag_array) {
            let _ = map.tag_array.insert_recursive(&multiplayer.tag_array, i.tag_index);
        }
    }

    let bitmaps_ce_map_o = match bitmaps_ce {
        Some(n) => Some(try!(ResourceMap::from_resource_map(n))),
        None => None
    };

    let sounds_ce_map_o = match sounds_ce {
        Some(n) => Some(try!(ResourceMap::from_resource_map(n))),
        None => None
    };

    'tag_loop : for t in map.tag_array.tags_mut() {
        if t.implicit {
            continue;
        }
        if t.tag_class.0 == 0x6269746D {
            if let Some(bitmaps_ce_map) = bitmaps_ce_map_o.as_ref() {
                let tag_data = t.data.as_ref().unwrap().to_owned();
                let t_count = LittleEndian::read_u32(&tag_data[0x60..]) as usize;
                let t_offset = LittleEndian::read_u32(&tag_data[0x64..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
                let t_data = &tag_data[t_offset..];
                'resource_loop_bitm : for r in 0..bitmaps_ce_map.resources.len() {
                    let resource = &bitmaps_ce_map.resources[r];
                    if resource.name.ends_with("__pixels") {
                        continue;
                    }
                    let r_count = LittleEndian::read_u32(&resource.data[0x60..]) as usize;
                    if r_count != t_count {
                        continue;
                    }
                    let r_offset = LittleEndian::read_u32(&resource.data[0x64..]) as usize;
                    let r_data = &resource.data[r_offset ..];
                    for b in 0..t_count {
                        let r_bitmap = &r_data[b * 48 .. (b + 1) * 48];
                        let t_bitmap = &t_data[b * 48 .. (b + 1) * 48];
                        let r_bitmap_size = LittleEndian::read_u32(&r_bitmap[0x1C..]) as usize;
                        let t_bitmap_size = LittleEndian::read_u32(&t_bitmap[0x1C..]) as usize;
                        if t_bitmap_size != r_bitmap_size {
                            continue 'resource_loop_bitm;
                        }
                        let r_bitmap_offset = LittleEndian::read_u32(&r_bitmap[0x18..]) as usize;
                        let t_bitmap_offset = LittleEndian::read_u32(&t_bitmap[0x18..]) as usize;
                        let t_bitmap_data = if t_bitmap[0xF] & 1 == 1 {
                            &bitmaps_pc[t_bitmap_offset .. t_bitmap_offset + t_bitmap_size]
                        }
                        else {
                            &t.asset_data.as_ref().unwrap()[t_bitmap_offset .. t_bitmap_offset + t_bitmap_size]
                        };
                        let r_bitmap_data = &bitmaps_ce.as_ref().unwrap()[r_bitmap_offset .. r_bitmap_offset + r_bitmap_size];

                        if t_bitmap_data != r_bitmap_data {
                            continue 'resource_loop_bitm;
                        }
                    }
                    t.asset_data = None;
                    t.data = None;
                    t.memory_address = None;
                    t.implicit = true;
                    t.resource_index = Some(r as u32);
                    continue 'tag_loop;
                }
            }

            let mut tag_data = t.data.as_mut().unwrap();
            let t_count = LittleEndian::read_u32(&tag_data[0x60..]) as usize;
            let t_offset = LittleEndian::read_u32(&tag_data[0x64..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
            let mut t_data = &mut tag_data[t_offset..];
            for b in 0..t_count {
                let mut t_bitmap = &mut t_data[b * 48 .. (b + 1) * 48];
                if t_bitmap[0xF] & 1 == 0 {
                    continue;
                }
                let t_bitmap_size = LittleEndian::read_u32(&t_bitmap[0x1C..]) as usize;
                let t_bitmap_offset = LittleEndian::read_u32(&t_bitmap[0x18..]) as usize;
                if t.asset_data.is_none() {
                    t.asset_data = Some(Vec::new())
                }
                LittleEndian::write_u32(&mut t_bitmap[0x18..], t.asset_data.as_ref().unwrap().len() as u32);
                t_bitmap[0xF] = 0;
                t.asset_data.as_mut().unwrap().extend_from_slice(&bitmaps_pc[t_bitmap_offset .. t_bitmap_offset + t_bitmap_size]);
            }
        }
        else if t.tag_class.0 == 0x736E6421 {
            if let Some(sounds_ce_map) = sounds_ce_map_o.as_ref() {
                let tag_data = t.data.as_mut().unwrap();
                let count = LittleEndian::read_u32(&tag_data[0x98..]) as usize;
                if count == 0 {
                    continue;
                }
                let offset = LittleEndian::read_u32(&tag_data[0x98 + 4..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
                let ranges = &tag_data[offset .. offset + count * 0x48].to_owned();
                'resource_loop_snd : for r in 0..sounds_ce_map.resources.len() {
                    let resource = &sounds_ce_map.resources[r];
                    if resource.name.ends_with("__samples") {
                        continue;
                    }
                    let r_count = LittleEndian::read_u32(&resource.data[0x98..]) as usize;
                    if r_count != count {
                        continue;
                    }
                    let r_offset = 0xA4;

                    let r_ranges = &resource.data[r_offset .. r_offset + count * 0x48];
                    for r in 0..r_count {
                        let t_range = &ranges[r * 0x48 .. (r + 1) * 0x48];
                        let r_range = &r_ranges[r * 0x48 .. (r + 1) * 0x48];
                        let t_permutation_count = LittleEndian::read_u32(&t_range[0x3C..]) as usize;
                        let r_permutation_count = LittleEndian::read_u32(&r_range[0x3C..]) as usize;
                        if t_permutation_count != r_permutation_count {
                            continue 'resource_loop_snd;
                        }
                        let t_permutation_offset = LittleEndian::read_u32(&t_range[0x3C+4..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
                        let r_permutation_offset = LittleEndian::read_u32(&r_range[0x3C+4..]) as usize + r_offset;
                        let t_permutations = &tag_data[t_permutation_offset .. t_permutation_offset + t_permutation_count * 124];
                        let r_permutations = &resource.data[r_permutation_offset .. r_permutation_offset + r_permutation_count * 124];
                        for p in 0..t_permutation_count {
                            let t_permutation = &t_permutations[p * 124 .. (p + 1) * 124];
                            let r_permutation = &r_permutations[p * 124 .. (p + 1) * 124];

                            let t_data_size = LittleEndian::read_u32(&t_permutation[0x40..]) as usize;
                            let r_data_size = LittleEndian::read_u32(&r_permutation[0x40..]) as usize;
                            if t_data_size != r_data_size {
                                continue 'resource_loop_snd;
                            }
                            let t_data_offset = LittleEndian::read_u32(&t_permutation[0x48..]) as usize;
                            let r_data_offset = LittleEndian::read_u32(&r_permutation[0x48..]) as usize;

                            let t_sound_data = if t_permutation[0x44] & 1 == 1 {
                                &sounds_pc[t_data_offset .. t_data_offset + t_data_size]
                            }
                            else {
                                &t.asset_data.as_ref().unwrap()[t_data_offset .. t_data_offset + t_data_size]
                            };
                            let r_sound_data = &sounds_ce.as_ref().unwrap()[r_data_offset .. r_data_offset + r_data_size];
                            if r_sound_data != t_sound_data {
                                continue 'resource_loop_snd;
                            }
                        }
                        t.asset_data = None;
                        t.implicit = true;
                        t.tag_path = resource.name.clone();
                        continue 'tag_loop;
                    }
                }
            }
            let mut tag_data = t.data.as_mut().unwrap();
            let count = LittleEndian::read_u32(&tag_data[0x98..]) as usize;
            let offset = LittleEndian::read_u32(&tag_data[0x98 + 4..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
            let ranges = &tag_data[offset .. offset + count * 0x48].to_owned();

            for i in 0..count {
                let range = &ranges[i * 0x48 .. (i+1)* 0x48];
                let permutations_count = LittleEndian::read_u32(&range[0x3C..]) as usize;
                let permutations_offset = LittleEndian::read_u32(&range[0x3C+4..]) as usize - *t.memory_address.as_ref().unwrap() as usize;
                let mut permutations = &mut tag_data[permutations_offset .. permutations_offset + permutations_count * 124];
                for p in 0..permutations_count {
                    let mut permutation = &mut permutations[p * 124 .. (p+1) * 124];
                    if permutation[0x44] & 1 == 0 {
                        continue;
                    }
                    let data_offset = LittleEndian::read_u32(&permutation[0x48..]) as usize;
                    let data_size = LittleEndian::read_u32(&permutation[0x40..]) as usize;
                    if t.asset_data.is_none() {
                        t.asset_data = Some(Vec::new())
                    }
                    LittleEndian::write_u32(&mut permutation[0x48..], t.asset_data.as_ref().unwrap().len() as u32);
                    permutation[0x44] = 0;
                    t.asset_data.as_mut().unwrap().extend_from_slice(&sounds_pc[data_offset .. data_offset + data_size]);
                }
            }
        }
    }

    map.kind.0 = Game::HaloCustomEdition;

    map.as_cache_file()
}
