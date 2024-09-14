CREATE TABLE sticker (
    uuid TEXT PRIMARY KEY,
    markdown TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '',
    pos_x INT NOT NULL,
    pos_y INT NOT NULL,
    height INT NOT NULL,
    width INT NOT NULL,
    pinned INT NOT NULL DEFAULT 0,
    archived INT NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
);

CREATE TRIGGER sticker_updated_trigger AFTER UPDATE ON sticker
    BEGIN
        UPDATE sticker SET updated_at=datetime('now', 'localtime') WHERE rowid = new.rowid;
    END;