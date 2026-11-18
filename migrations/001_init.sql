CREATE TABLE IF NOT EXISTS inventory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    set_code TEXT NOT NULL,
    collector_number TEXT NOT NULL,
    lang TEXT NOT NULL DEFAULT 'en',
    is_foil INTEGER NOT NULL DEFAULT 0,
    name TEXT NOT NULL,
    name_en TEXT,
    name_es TEXT,
    rarity TEXT,
    colors TEXT NOT NULL DEFAULT '[]',
    price_usd REAL,
    image_url TEXT,
    bin_number INTEGER,
    quantity INTEGER NOT NULL DEFAULT 1,
    last_updated TEXT NOT NULL DEFAULT (datetime('now')),
    UNIQUE(set_code, collector_number, lang, is_foil)
);

CREATE TABLE IF NOT EXISTS scan_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    inventory_id INTEGER NOT NULL,
    source TEXT NOT NULL DEFAULT 'unknown',
    bin_number INTEGER,
    scanned_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (inventory_id) REFERENCES inventory(id) ON DELETE CASCADE
);