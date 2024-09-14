CREATE TABLE sticker (
    uuid TEXT PRIMARY KEY,
    markdown TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '',
    pos_x INT NOT NULL,
    pos_y INT NOT NULL,
    height INT NOT NULL,
    width INT NOT NULL,
    pinned INT NOT NULL DEFAULT 0,
    archived INT NOT NULL DEFAULT 0
)