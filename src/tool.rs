use crate::crate_prelude::*;
use crate::assets::EditorAssets;
use crate::ui::tooltip::TooltipText;
use std::ops::{BitOr, BitOrAssign};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[derive(Reflect, FromReflect)]
#[derive(enum_iterator::Sequence)]
#[repr(u8)]
pub enum Tool {
    #[default]
    SelectEntities = 0,
    Translation = 1,
    // tilemap tools
    SelectTilemap = 16,
}

impl Tool {
    pub(crate) fn icon(self, assets: &EditorAssets) -> Handle<Image> {
        match self {
            Tool::SelectEntities => assets.image_icon_tool_selectentities.clone(),
            Tool::Translation => assets.image_icon_tool_translation.clone(),
            Tool::SelectTilemap => assets.image_icon_tool_selecttilemap.clone(),
        }
    }

    pub(crate) fn tooltip(self) -> TooltipText {
        match self {
            Tool::SelectEntities => TooltipText {
                title: "Select Entities".into(),
                text: "Click on entities to select them.\nThen, use other tools to manipulate the selected entities.".into(),
            },
            Tool::Translation => TooltipText {
                title: "Move/Translate (Transform Editing)".into(),
                text: "Move entities with the mouse, changing the translation of their Transform.".into(),
            },
            Tool::SelectTilemap => TooltipText {
                title: "Select the Active Tilemap".into(),
                text: "Tilemap editing tools will operate on the currently selected tilemap.".into(),
            },
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
        tools: impl Into<Tools>,
    ) -> Self {
        let tools = tools.into();
        self.run_if(move |state: Res<CurrentState<Tool>>| {
            tools.contains(state.0)
        })
    }
}

impl<T: ConditionHelpers> RunConditionToolsExt for T {}
