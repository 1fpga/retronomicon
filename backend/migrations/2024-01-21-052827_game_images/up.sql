-- Your SQL goes here

CREATE TABLE game_images
(
    id         SERIAL PRIMARY KEY NOT NULL,
    game_id    integer            NOT NULL REFERENCES "games" ("id") ON DELETE CASCADE,
    image_name varchar(255)       NOT NULL,
    width      integer            NOT NULL,
    height     integer            NOT NULL,
    mime_type  varchar(255)       NOT NULL,
    CONSTRAINT game_images_game_id_image_name_key UNIQUE (game_id, image_name)
);

COMMENT ON TABLE game_images IS 'Images related to games. The image_name field is the name of the image file.';

CREATE TABLE game_image_tags
(
    game_image_id integer NOT NULL REFERENCES "game_images" ("id") ON DELETE CASCADE,
    tag_id        integer NOT NULL REFERENCES "tags" ("id") ON DELETE CASCADE,
    CONSTRAINT game_image_tags_pkey PRIMARY KEY (game_image_id, tag_id)
);
