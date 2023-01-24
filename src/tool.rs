use crate::crate_prelude::*;
use crate::assets::EditorAssets;
use std::ops::{BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Reflect, FromReflect)]
#[derive(enum_iterator::Sequence)]
#[repr(u8)]
pub enum Tool {
    #[default]
    SelectEntities = 0,
    // tilemap tools
    SelectTilemap = 16,
}

impl Tool {
    pub(crate) fn icon(self, assets: &EditorAssets) -> Handle<Image> {
        match self {
            Tool::SelectEntities => assets.image_icon_tool_selectentities.clone(),
            Tool::SelectTilemap => assets.image_icon_tool_selecttilemap.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tools(u64);

impl Tools {
    pub fn contains(self, tool: Tool) -> bool {
        self.0 & (1 << tool as u8) != 0
    }
}

impl BitOr<Tool> for Tools {
    type Output = Tools;
    fn bitor(self, rhs: Tool) -> Self::Output {
        Tools(self.0 | (1 << rhs as u8))
    }
}

impl BitOr<Tool> for Tool {
    type Output = Tools;
    fn bitor(self, rhs: Tool) -> Self::Output {
        Tools((1 << self as u8) | (1 << rhs as u8))
    }
}

impl BitOr<Tools> for Tool {
    type Output = Tools;
    fn bitor(self, rhs: Tools) -> Self::Output {
        Tools(rhs.0 | (1 << self as u8))
    }
}

impl BitOr<Tools> for Tools {
    type Output = Tools;
    fn bitor(self, rhs: Tools) -> Self::Output {
        Tools(self.0 | rhs.0)
    }
}

impl BitOrAssign<Tool> for Tools {
    fn bitor_assign(&mut self, rhs: Tool) {
        self.0 |= 1 << rhs as u8;
    }
}

impl From<Tool> for Tools {
    fn from(value: Tool) -> Self {
        Tools(1 << value as u8)
    }
}

pub trait RunConditionToolsExt: ConditionHelpers {
    fn run_for_tools(
        self,
        tools: Tools,
    ) -> Self {
        self.run_if(move |state: Res<CurrentState<Tool>>| {
            tools.contains(state.0)
        })
    }
}

impl<T: ConditionHelpers> RunConditionToolsExt for T {}
