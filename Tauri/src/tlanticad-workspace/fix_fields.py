import sys

with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'r') as f:
    text = f.read()

text = text.replace("designs: Vec::new(),\n            status:", "designs: Vec::new(),\n            technician: None,\n            is_deleted: false,\n            global_shade: None,\n            antagonist_scan_mode: None,\n            is_imported: false,\n            status:")

with open('/Users/juliocesar/Desktop/gesto/crates/tlanticad-db/src/repository.rs', 'w') as f:
    f.write(text)
