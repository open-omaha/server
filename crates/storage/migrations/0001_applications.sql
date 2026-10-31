CREATE TABLE applications (
    id TEXT PRIMARY KEY NOT NULL CHECK (id <> ''),
    omaha_app_id TEXT NOT NULL COLLATE NOCASE UNIQUE CHECK (omaha_app_id <> ''),
    display_name TEXT NOT NULL,
    version_scheme TEXT NOT NULL CHECK (version_scheme IN ('CHROMIUM', 'SEMVER', 'OMAHA_LEGACY')),
    cup_policy TEXT NOT NULL CHECK (cup_policy IN ('REQUIRED', 'OPTIONAL'))
);
