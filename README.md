# WIP: 2D level editor for Bevy

The idea is that this editor will be useful for creating 2D game levels in a
Bevy-native way.

If you need an editor for 3D projects, consider `bevy_editor_pls`.

Designed with `iyes_loopless`.
Your project must use `iyes_loopless` app states to be compatible.

## Assets

The editor needs an asset pack (fonts, toolbar icons, etc.). A default one is
provided in the `assets` folder in this repo. You need to copy it into your
project, to use the editor.

Feel free to replace any of the assets if you want to "theme"/"skin" the editor
to your preferences.

Assets are managed using `bevy_asset_loader`, using its "dynamic assets" feature.
All the filenames / asset paths are specified via the `iyes2d_editor.assets` file.
If you want to rename or reorganize some asset files, just edit that file.

If you also use `bevy_asset_loader` in your project, you can add the editor's
`AssetCollection` to your loading state, if you want to control when the assets
get loaded (such as loading them during the same loading screen as your game's
assets). This is optional. If you do nothing, the editor will detect that the
resource is missing, and take care of loading its assets by itself.

## Design Scope

Support workflows for the following:
  - [ ] Bevy sprite entities
  - [ ] `bevy_ecs_tilemap` tilemaps (optional)
  - [ ] `bevy_prototype_lyon` 2D shapes (optional)
  - [ ] `bevy_rapier` 2D collision shapes (optional)
  - [ ] `bevy_sepax2d` 2D collision shapes (optional)

Transform editing tools:
  - [ ] Translation / Position / moving entities around
  - [ ] Rotation
  - [ ] Scale
  - [ ] Snap to grid

Sprite tools:
  - [ ] Place sprite entities onto the map
  - [ ] Visual selection and cursor

Basic shape editing tools:
  - [ ] Place and resize rectangles
  - [ ] Place and resize circles
  - [ ] Can be used for arbitrary component types: colliders, renderable shapes, etc.
  - [ ] Visual selection and cursor

Tilemap editing tools:
  - [ ] The ability to paint regions with different properties using the mouse cursor
  - [ ] The ability to work with multiple tilemaps and layers
  - [ ] Autogeneration of colliders and other abstract rectangular regions based on the tiles
  - [ ] Visual selection and cursor

Autotiling (based on crate `iyes2d_autotilemap`):
  - [ ] Automatic adding of ECS components to tiles based on rules
  - [ ] Automatic selection of texture ids for the tiles based on rules
  - [ ] Rules editor for visually creating said rules

ECS editing:
  - [ ] Add/remove components on tilemap tile entities
  - [ ] Add/remove components on sprite entities
  - [ ] Add/remove components on shape entities

Scene Support:
  - [ ] Export the level as a Bevy Scene file, containing just the data needed to load and initialize things (using `iyes_scene_tools`)
  - [ ] "Blueprints pattern": spawn (or load from scene) partial entities and complete their initialization (flesh them out with additional components) from code/systems (using `iyes_blueprints`)

UI:
  - [x] Toolbar for selecting different tools
  - [x] Tooltips
  - [x] Window Management: floating windows for UI panels
  - [ ] Blueprint list
  - [ ] Component selection
  - [ ] Hover previews

## Licenses

All the relevant license texts are available as plain text files in this repository.

The software (all source code) in this repository is distributed under a dual MIT/Apache-2 license.

The following assets in this repository are foreign, and use different licenses:
 - `assets/iyes2d_editor/font/Ubuntu*.ttf`: the Ubuntu font, under the Ubuntu font license

All other assets in this repository were created specifically for this project and are distributed under CC0.
