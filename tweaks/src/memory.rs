use crate::{Tweak, TweakError, TweakStatus};

pub struct EmptyStandbyListTweak;

impl Tweak for EmptyStandbyListTweak {
    fn id(&self) -> &'static str {
        "empty_standby_list"
    }

    fn name(&self) -> &'static str {
        "Limpeza da Standby List (RAM)"
    }

    fn description(&self) -> &'static str {
        "Esvazia a lista de memória em standby do Windows, liberando memória RAM em cache sem afetar working sets."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        Ok(TweakStatus::Unknown)
    }

    fn apply(&self) -> Result<(), TweakError> {
        windows_api::empty_standby_list()?;
        Ok(())
    }

    fn rollback(&self) -> Result<(), TweakError> {
        Ok(())
    }
}

pub fn empty_standby_list() -> Result<(), TweakError> {
    windows_api::empty_standby_list()?;
    Ok(())
}
