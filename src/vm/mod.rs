#[derive(Clone, Debug, Default)]
pub struct GasMeter { pub limit: u64, pub used: u64 }

impl GasMeter {
    pub fn new(limit: u64) -> Self { Self { limit, used: 0 } }
    pub fn charge(&mut self, amount: u64) -> Result<(), String> {
        self.used = self.used.checked_add(amount).ok_or_else(|| "gas overflow".to_string())?;
        if self.used > self.limit { return Err("out of gas".to_string()); }
        Ok(())
    }
}
