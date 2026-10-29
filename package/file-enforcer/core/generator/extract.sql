-- Extracts package availability and per-manager package names
-- for all projects that exist in both Fedora and Ubuntu.
--
-- Output: JSON array where each element is:
--   { "effname": "...", "repos": { "manager": "pkgname", ... } }
--
-- Manager mapping:
--   fedora_43           -> dnf
--   ubuntu_26_04        -> apt
--   arch                -> pacman
--   alpine_3_23         -> apk
--   opensuse_tumbleweed -> zypper
--   homebrew            -> brew
--   chocolatey          -> choco
--   scoop               -> scoop

WITH target_repos AS (
    SELECT unnest(ARRAY[
        'fedora_43', 'ubuntu_26_04', 'arch', 'alpine_3_23',
        'opensuse_tumbleweed', 'homebrew', 'chocolatey', 'scoop'
    ]) AS repo
),
-- Projects that exist in BOTH fedora and ubuntu (required baseline)
required_projects AS (
    SELECT effname
    FROM packages
    WHERE repo = 'fedora_43'
    INTERSECT
    SELECT effname
    FROM packages
    WHERE repo = 'ubuntu_26_04'
),
-- For each (effname, repo) pair, pick the best installable package name.
-- Priority: binnames[1] > binname > srcname > visiblename.
-- For repos with multiple entries per project (e.g. alpine sub-packages),
-- prefer the entry where srcname or binname matches the effname.
ranked AS (
    SELECT DISTINCT ON (p.effname, p.repo)
        p.effname,
        p.repo,
        COALESCE(p.binnames[1], p.binname, p.srcname, p.visiblename) AS pkgname
    FROM packages p
    INNER JOIN required_projects rp ON rp.effname = p.effname
    INNER JOIN target_repos tr ON tr.repo = p.repo
    ORDER BY p.effname, p.repo,
        -- prefer entry where name matches effname (base package)
        CASE WHEN COALESCE(p.binname, p.srcname) = p.effname THEN 0 ELSE 1 END,
        -- then prefer entries with binname set
        CASE WHEN p.binname IS NOT NULL THEN 0 ELSE 1 END
),
-- Map repo names to our manager names
mapped AS (
    SELECT
        effname,
        CASE repo
            WHEN 'fedora_43'           THEN 'dnf'
            WHEN 'ubuntu_26_04'        THEN 'apt'
            WHEN 'arch'                THEN 'pacman'
            WHEN 'alpine_3_23'         THEN 'apk'
            WHEN 'opensuse_tumbleweed' THEN 'zypper'
            WHEN 'homebrew'            THEN 'brew'
            WHEN 'chocolatey'          THEN 'choco'
            WHEN 'scoop'              THEN 'scoop'
        END AS manager,
        pkgname
    FROM ranked
),
-- Aggregate per-manager names into a JSON object per project
aggregated AS (
    SELECT
        effname,
        jsonb_object_agg(manager, pkgname) AS repos
    FROM mapped
    GROUP BY effname
)
SELECT jsonb_agg(
    jsonb_build_object('effname', effname, 'repos', repos)
    ORDER BY effname
)
FROM aggregated;
