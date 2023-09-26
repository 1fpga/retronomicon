-- Add root user
INSERT INTO "teams" VALUES(1,
                            'root', 'root',
                            'The root team which has administrative right.',
                            '{
                              "github": "https://github.com/golem-fpga/retronomicon"
                            }'::jsonb
                           );
