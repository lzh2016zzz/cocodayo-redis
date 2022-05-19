use std::{
    fs::File,
    io::{self, BufRead},
};

pub fn banner_show(file_name: &str) {
    let file = File::open(file_name);
    match file {
        Ok(f) => {
            let reader = io::BufReader::new(f);
            for f in reader.lines() {
                match f {
                    Ok(value) => log::info!("{}", value),
                    Err(_) => {}
                }
            }
        }
        Err(err) => {
            if file_name != "" {
                log::info!("Failed to load banner file ,name : {},{}", file_name, err);
            }

            let banner_str = r#"
            dP""b8  dP"Yb   dP""b8  dP"Yb  8888b.     db    Yb  dP  dP"Yb      88""Yb 888888 8888b.  88 .dP"Y8 
            dP   `" dP   Yb dP   `" dP   Yb  8I  Yb   dPYb    YbdP  dP   Yb     88__dP 88__    8I  Yb 88 `Ybo." 
            Yb      Yb   dP Yb      Yb   dP  8I  dY  dP__Yb    8P   Yb   dP     88"Yb  88""    8I  dY 88 o.`Y8b 
             YboodP  YbodP   YboodP  YbodP  8888Y"  dP""""Yb  dP     YbodP      88  Yb 888888 8888Y"  88 8bodP' 
             "#;
            log::info!("{}", banner_str)
        }
    }
}
