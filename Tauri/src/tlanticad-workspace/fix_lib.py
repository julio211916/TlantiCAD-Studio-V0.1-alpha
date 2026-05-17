import sys
with open('/Users/juliocesar/Desktop/gesto/apps/tlantidb/mac/src-tauri/src/lib.rs', 'r') as f:
    text = f.read()

text = text.replace("commands::db::create_project,", "commands::db::create_project,\n            commands::db::get_project,\n            commands::db::delete_project,\n            commands::db::launch_cad,")

with open('/Users/juliocesar/Desktop/gesto/apps/tlantidb/mac/src-tauri/src/lib.rs', 'w') as f:
    f.write(text)
