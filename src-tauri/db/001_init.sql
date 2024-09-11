CREATE TABLE sticker (
    uuid TEXT PRIMARY KEY,
    markdown TEXT NOT NULL,
    pos_x INT NOT NULL,
    pos_y INT NOT NULL,
    height INT NOT NULL,
    width INT NOT NULL,
    archived INT NOT NULL DEFAULT 0
)