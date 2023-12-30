use eyre::Result;
use reqwest;
use rosu_pp::Beatmap;
use std::{env, fs::File, io::Write, path::Path};

pub async fn get_beatmap(id: u32) -> Result<Beatmap> {
    let dir = env::var("OSU_MAP_PATH")?;
    let osu_path = format!("{dir}{id}.osu");
    let path = Path::new(&osu_path);
    if path.exists() {
        Ok(Beatmap::from_path(path)?)
    } else {
        let response = reqwest::get(format!("https://osu.ppy.sh/osu/{id}"))
            .await?
            .text()
            .await?;
        let mut file = File::create(&format!("{dir}{id}.osu"))?;
        file.write_all(response.as_bytes())?;
        Ok(Beatmap::from_path(path)?)
    }
}
