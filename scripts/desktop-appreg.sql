CREATE TABLE IF NOT EXISTS mimetypes (
    ext TEXT,
    mimetype TEXT
);
CREATE TABLE IF NOT EXISTS extenstions (
    ext TEXT,
    mimetype TEXT
);
CREATE TABLE IF NOT EXISTS handlerIds (
    handlerId TEXT
);
CREATE TABLE IF NOT EXISTS properties (
    handlerId TEXT,
    name TEXT,
    value TEXT
);
CREATE TABLE IF NOT EXISTS associations (
    interface TEXT,
    handlerId TEXT,
    contentId TEXT,
    defaultAssoc TEXT
);
