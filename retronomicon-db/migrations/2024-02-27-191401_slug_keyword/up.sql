ALTER DOMAIN slug ADD
    CONSTRAINT slug_constraint_keywords
        CHECK (
            value != ''
                AND value NOT IN (
                                  'new',
                                  'edit',
                                  'delete',
                                  'latest',
                                  'popular',
                                  'invalid',
                                  'all'
                )
            );
