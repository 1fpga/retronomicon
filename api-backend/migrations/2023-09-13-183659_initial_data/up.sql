-- Add root user
INSERT INTO "groups" VALUES(1,
                            'root', 'root',
                            'The root group which has administrative right.',
                            '{
                              "github": "https://github.com/golem-fpga/retronomicon"
                            }'::jsonb
                           );
