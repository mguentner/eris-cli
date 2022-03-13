use clap::App;
use clap::Arg;
use eris_rs::types::BlockStorageError;
use eris_rs::types::BlockStorageErrorKind;
use eris_rs::types::Reference;
use eris_rs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io;
use std::path::PathBuf;


fn reference_to_path(eris_store: &str, reference: eris_rs::types::Reference) -> PathBuf {
    let base32_alphabet = base32::Alphabet::RFC4648 { padding: false };
    let ref_encoded = base32::encode(base32_alphabet, &reference);
    let first_two_bytes = &ref_encoded[0..2];
    let second_two_bytes = &ref_encoded[2..4];
    // create a simple leveled folder to make look ups faster
    [eris_store, &first_two_bytes, &second_two_bytes, &ref_encoded]
        .iter()
        .collect()
}

fn encode(
    reader: &mut dyn std::io::Read,
    block_size: eris_rs::types::BlockSize,
    convergence_secret: &[u8],
    target_directory: String,
) -> Result<eris_rs::types::ReadCapability, std::io::Error> {
    let write_fn = move |block_with_reference: eris_rs::types::BlockWithReference| -> Result<usize, eris_rs::types::BlockStorageError> {
                let path = reference_to_path(&target_directory, block_with_reference.reference);
                let parent = path.parent().unwrap();
                match path.exists() {
                    true => Ok(block_with_reference.block.len()),
                    false => match std::fs::create_dir_all(parent) {
                        Ok(_) => match File::create(path) {
                            Ok(mut f) => {
                                f.write(&block_with_reference.block)
                            },
                            Err(e) => Err(e)
                        },
                        Err(e) => Err(e)
                    }
                }
            };
    return eris_rs::encode::encode(reader, convergence_secret, block_size, &write_fn);
}

fn decode(
    eris_store_dir: String,
    target: &mut dyn std::io::Write,
    read_capability: eris_rs::types::ReadCapability,
) -> Result<usize, std::io::Error> {
    let eris_get_fn = move |reference: Reference| -> Result<Vec<u8>, BlockStorageError> {
        let path = reference_to_path(&eris_store_dir, reference);
        match path.exists() {
            true => match File::open(path) {
                Ok(mut f) => {
                    let mut buf = Vec::new();
                    match f.read_to_end(&mut buf) {
                        Ok(_) => Ok(buf),
                        Err(e) => Err(e)
                    }
                },
                Err(e) => Err(e)
            },
            false => Err(BlockStorageError::new(BlockStorageErrorKind::NotFound, "not found in eris store"))
        }
    };

    return eris_rs::decode::decode(read_capability, target, &eris_get_fn);
}

fn main() {
    let m = App::new("eris-cli")
        .arg(Arg::new("file").required(true).short('f').long("file").takes_value(true).value_name("FILE").help("File to read from or write to"))
        .arg(Arg::new("store").required(true).short('s').long("store").takes_value(true).value_name("STORE_DIR").help("directory of the eris store"))
        .arg(Arg::new("encode").short('e').long("encode").takes_value(false).help("encode mode").conflicts_with("decode"))
        .arg(Arg::new("decode").short('d').long("decode").takes_value(false).help("decode mode").conflicts_with("encode"))
        .arg(Arg::new("convergence-secret").short('c').long("convergence-secret").takes_value(true).value_name("BASE32").help("the precomputed convergence secret in BASE32"))
        .arg(Arg::new("urn").short('u').long("urn").takes_value(true).value_name("urn").help("the urn to decode").conflicts_with("encode"))
        .arg(Arg::new("block-size").short('b').takes_value(true).possible_values(["1","32"]).help("block size to use").conflicts_with("decode").required_unless_present("decode"))
        .get_matches();

    let file_name: String = m.value_of_t_or_exit("file");
    let store_dir = m.value_of_t_or_exit("store");
    if m.is_present("encode") {
        let block_size = match m.value_of("block-size") {
            Some("1") => eris_rs::types::BlockSize::Size1KiB,
            Some("32") => eris_rs::types::BlockSize::Size32KiB,
            Some(b) => panic!("unexpected block size: {}", b),
            None => panic!("Expected a block-size")
        };
        let convergence_secret: [u8; 32] = match m.value_of("convergence-secret") {
            Some(cs) => {
                let base32_alphabet = base32::Alphabet::RFC4648 { padding: false };
                match base32::decode(base32_alphabet, cs) {
                    Some(v) => {
                        let mut result: [u8; 32] = Default::default();
                        if v.len() == eris_rs::constants::KEY_SIZE_BYTES {
                            result.copy_from_slice(&v);
                            result
                        } else {
                            panic!("convergence secret not of correct length")
                        }
                    },
                    None => {
                        panic!("Failed to parse convergence-secret")
                    }
                }
            },
            None => {
                let mut result: [u8; 32] = Default::default();
                result.fill(0);
                result
            }
        };
        if file_name == "-" {
            let mut stdin = io::stdin();
            match encode(&mut stdin, block_size, &convergence_secret, store_dir) {
                Ok(read_capability) => eprintln!("{}", read_capability.to_urn()),
                Err(e) => panic!("{}", e),
            }
        } else {
            let mut file = File::open(file_name).unwrap();
            match encode(&mut file, block_size, &convergence_secret, store_dir) {
                Ok(read_capability) => eprintln!("{}", read_capability.to_urn()),
                Err(e) => panic!("{}", e),
            }
        }
    } else {
        // decode
        let urn = m.value_of_t_or_exit("urn");
        let read_capability = match eris_rs::types::ReadCapability::from_urn(urn) {
            Some(v) => v,
            None => panic!("Not an urn")
        };
        if file_name == "-" {
            let stdout =  io::stdout();
            let mut handle = stdout.lock();
            match decode(store_dir, &mut handle, read_capability) {
                Ok(_) => {
                    // stdout is a BufWriter, a final flush is necessary
                    handle.flush().unwrap();
                    eprintln!("done.")
                },
                Err(e) => eprintln!("error while decoding: {}", e)
            }
        } else {
            match File::create(file_name) {
                Ok(mut f) => {
                    match decode(store_dir, &mut f, read_capability) {
                        Ok(_) => eprintln!("done."),
                        Err(e) => eprintln!("error while decoding: {}", e)
                    }
                },
                Err(e) => eprintln!("could not create file: {}", e)
            }
        }
    }
}
