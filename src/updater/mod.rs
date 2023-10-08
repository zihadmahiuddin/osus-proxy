#[derive(Default)]
pub struct Updater {
    client: reqwest::blocking::Client
}

impl Updater {
    pub fn check_for_updates(&self) -> color_eyre::Result<bool> {
        let resp = self.client.head("https://osus-proxy-update-server.vercel.app/api/handler").send()?;
        let executable_data = std::fs::read(std::env::current_exe()?)?;

        let hash = sha256::digest(executable_data);
        let remote_hash = resp.headers().get("X-Content-Hash");
        if let Some(remote_hash) = remote_hash {
            let remote_hash: Vec<&str> = remote_hash.to_str()?.split("sha256-").collect();
            if remote_hash.len() >= 2 {
                let remote_hash = remote_hash[1].to_string();
                return Ok(remote_hash != hash);
            }
            
        }
        
        Ok(false)
    }
}