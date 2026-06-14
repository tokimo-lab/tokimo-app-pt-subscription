use crate::pt_search::config::*;

/// Look up a site configuration by its `site_id`.
pub fn get_site_config(site_id: &str) -> Option<SiteConfig> {
    match site_id {
        "hdfans" => Some(hdfans()),
        "hdsky" => Some(hdsky()),
        "audiences" => Some(audiences()),
        "azusa" => Some(azusa()),
        "btschool" => Some(btschool()),
        "chdbits" => Some(chdbits()),
        "hddolby" => Some(hddolby()),
        "HDHome" => Some(hdhome()),
        "hhan" => Some(hhan()),
        "keepfrds" => Some(keepfrds()),
        "ourbits" => Some(ourbits()),
        "pterclub" => Some(pterclub()),
        "ptsbao" => Some(ptsbao()),
        "putao" => Some(putao()),
        "ssd" => Some(springsunday()),
        "ttg" => Some(totheglory()),
        "ultrahd" => Some(ultrahd()),
        "exoticaz" => Some(exoticaz()),
        "filelist" => Some(filelist()),
        "iptorrents" => Some(iptorrents()),
        "hares" => Some(hares()),
        "hdatmos" => Some(hdatmos()),
        "agsv" => Some(agsv()),
        "tjupt" => Some(tjupt()),
        "m-team" => Some(mteam()),
        "acgrip" => Some(acg()),
        "mikanani" => Some(mikanani()),
        "sukebei" => Some(sukebei()),
        _ => None,
    }
}

/// Return all known site IDs
pub fn all_site_ids() -> &'static [&'static str] {
    &[
        "hdfans",
        "hdsky",
        "audiences",
        "azusa",
        "btschool",
        "chdbits",
        "hddolby",
        "HDHome",
        "hhan",
        "keepfrds",
        "ourbits",
        "pterclub",
        "ptsbao",
        "putao",
        "ssd",
        "ttg",
        "ultrahd",
        "exoticaz",
        "filelist",
        "iptorrents",
        "hares",
        "hdatmos",
        "agsv",
        "tjupt",
        "m-team",
        "acgrip",
        "mikanani",
        "sukebei",
    ]
}

// ============================================================
// Shared defaults
// ============================================================

fn nexus_search_query() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "search",
            "{% if query.imdb_id %}{{query.imdb_id}}{%else%}{{query.keyword}}{% endif %}",
        ),
        ("incldead", "1"),
        ("spstate", "{% if query.free %}2{% else %}0{% endif %}"),
        ("search_area", "{% if query.imdb_id %}4{% else %}0{%endif%}"),
        ("search_mode", "0"),
        ("sort", "{{ query.sort }}"),
        ("type", "{{ query.type }}"),
        ("page", "{{ query.page }}"),
    ]
}

const NEXUS_ROW_SELECTOR: &str = "table.torrents > tr:has(table)";

fn nexus_id() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[href^="details.php?id="]"#),
        attribute: Some("href"),
        filters: vec![Filter::ReSearch(r"\d+", 0)],
        ..Default::default()
    }
}

fn nexus_title_default() -> FieldConfig {
    FieldConfig::selector(r#"a[href^="details.php?id="]"#)
}

fn nexus_title_optional() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[title][href^="details.php?id="]"#),
        attribute: Some("title"),
        ..Default::default()
    }
}

fn nexus_title() -> FieldConfig {
    FieldConfig::text_tpl(
        "{% if fields['title_optional'] %}{{ fields['title_optional'] }}{% else %}{{ fields['title_default'] }}{% endif %}",
    )
}

fn nexus_category() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[href^="?cat="]"#),
        attribute: Some("href"),
        filters: vec![Filter::Replace("?", ""), Filter::QueryString("cat")],
        ..Default::default()
    }
}

fn nexus_category_qs_only() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[href^="?cat="]"#),
        attribute: Some("href"),
        filters: vec![Filter::QueryString("cat")],
        ..Default::default()
    }
}

fn nexus_details() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[href^="details.php?id="]"#),
        attribute: Some("href"),
        ..Default::default()
    }
}

fn nexus_download() -> FieldConfig {
    FieldConfig {
        selector: Some(r#"a[href^="download.php?id="]"#),
        attribute: Some("href"),
        ..Default::default()
    }
}

fn nexus_imdbid() -> FieldConfig {
    FieldConfig {
        selector: Some("div.imdb_100 > a"),
        attribute: Some("href"),
        filters: vec![Filter::ReSearch(r"tt\d+", 0)],
        ..Default::default()
    }
}

fn nexus_date_elapsed() -> FieldConfig {
    FieldConfig {
        selector: Some("td:nth-child(4) > span[title]"),
        attribute: Some("title"),
        ..Default::default()
    }
}

fn nexus_date_added() -> FieldConfig {
    FieldConfig {
        selector: Some("td:nth-child(4):not(:has(span))"),
        ..Default::default()
    }
}

fn nexus_date() -> FieldConfig {
    FieldConfig::text_tpl(
        "{% if fields['date_elapsed'] or fields['date_added'] %}{{ fields['date_elapsed'] if fields['date_elapsed'] else fields['date_added'] }}{% else %}now{% endif %}",
    )
}

fn nexus_size() -> FieldConfig {
    FieldConfig::selector("td:nth-child(5)")
}

fn nexus_seeders() -> FieldConfig {
    FieldConfig::selector("td:nth-child(6)")
}

fn nexus_leechers() -> FieldConfig {
    FieldConfig::selector("td:nth-child(7)")
}

fn nexus_grabs() -> FieldConfig {
    FieldConfig::selector("td:nth-child(8)")
}

fn nexus_dl_factor() -> FieldConfig {
    FieldConfig {
        case: vec![
            ("img.pro_free", "0"),
            ("img.pro_free2up", "0"),
            ("img.pro_50pctdown", "0.5"),
            ("img.pro_50pctdown2up", "0.5"),
            ("img.pro_30pctdown", "0.3"),
            ("*", "1"),
        ],
        ..Default::default()
    }
}

fn nexus_ul_factor() -> FieldConfig {
    FieldConfig {
        case: vec![
            ("img.pro_50pctdown2up", "2"),
            ("img.pro_free2up", "2"),
            ("img.pro_2up", "2"),
            ("*", "1"),
        ],
        ..Default::default()
    }
}

fn nexus_free_deadline() -> FieldConfig {
    FieldConfig {
        default_value: Some("{% if fields['downloadvolumefactor']==0 %}{{max_time}}{% endif%}"),
        selector: Some("img.pro_free,img.pro_free2up"),
        attribute: Some("onmouseover"),
        filters: vec![
            Filter::ReSearch(r"\d+-\d+-\d+ \d+:\d+:\d+", 0),
            Filter::DateParse("%Y-%m-%d %H:%M:%S"),
        ],
        ..Default::default()
    }
}

/// Standard `NexusPHP` field map with optional overrides
fn nexus_defaults(overrides: NexusOverrides) -> FieldMap {
    let category = if overrides.category_qs_only {
        Some(nexus_category_qs_only())
    } else {
        Some(nexus_category())
    };

    FieldMap {
        id: nexus_id(),
        title_default: nexus_title_default(),
        title_optional: nexus_title_optional(),
        title: nexus_title(),
        description: overrides.description,
        category,
        details: Some(nexus_details()),
        download: overrides.download.unwrap_or_else(nexus_download),
        date_elapsed: Some(nexus_date_elapsed()),
        date_added: Some(nexus_date_added()),
        date: Some(nexus_date()),
        size: nexus_size(),
        seeders: nexus_seeders(),
        leechers: nexus_leechers(),
        grabs: Some(nexus_grabs()),
        downloadvolumefactor: nexus_dl_factor(),
        uploadvolumefactor: nexus_ul_factor(),
        free_deadline: Some(overrides.free_deadline.unwrap_or_else(nexus_free_deadline)),
        imdbid: if overrides.has_imdb { Some(nexus_imdbid()) } else { None },
        extra: overrides.extra,
    }
}

#[derive(Default)]
struct NexusOverrides {
    description: Option<FieldConfig>,
    download: Option<FieldConfig>,
    free_deadline: Option<FieldConfig>,
    has_imdb: bool,
    category_qs_only: bool,
    extra: Vec<(&'static str, FieldConfig)>,
}

fn desc_tags_subject() -> FieldConfig {
    FieldConfig::text_tpl(
        "{% if fields['tags'] %}{{ fields['subject']+' '+fields['tags'] }}{% else %}{{ fields['subject'] }}{% endif %}",
    )
}

fn desc_embedded_contents() -> FieldConfig {
    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded")
}

fn desc_torrentname_contents() -> FieldConfig {
    FieldConfig::selector("table.torrentname > tr > td.embedded")
}

fn desc_torrentname_td() -> FieldConfig {
    FieldConfig::selector("table.torrentname > td")
}

fn tags_field_subtitle_div() -> FieldConfig {
    FieldConfig::selector("font.subtitle > div")
}

fn subject_field_subtitle() -> FieldConfig {
    FieldConfig::selector("font.subtitle")
}

// ============================================================
// Individual site configurations
// ============================================================

fn hdfans() -> SiteConfig {
    SiteConfig {
        site_id: "hdfans",
        name: "红豆饭",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            free_deadline: Some(FieldConfig {
                default_value: Some("{% if fields['downloadvolumefactor']==0 %}{{max_time}}{% endif%}"),
                selector: Some(r#"td[class="embedded"] > font > span[title]"#),
                attribute: Some("title"),
                filters: vec![Filter::DateParse("%Y-%m-%d %H:%M:%S")],
                ..Default::default()
            }),
            ..Default::default()
        }),
    }
}

fn hdsky() -> SiteConfig {
    SiteConfig {
        site_id: "hdsky",
        name: "天空",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: true,
            download: Some(FieldConfig {
                selector: Some(r#"form[action*="/download.php?id="]"#),
                attribute: Some("action"),
                ..Default::default()
            }),
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig {
                        selector: Some("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                        filters: vec![Filter::Replace("[优惠剩余时间：]", "")],
                        ..Default::default()
                    },
                ),
            ],
            ..Default::default()
        }),
    }
}

fn audiences() -> SiteConfig {
    SiteConfig {
        site_id: "audiences",
        name: "观众",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: true,
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                ),
            ],
            ..Default::default()
        }),
    }
}

fn azusa() -> SiteConfig {
    SiteConfig {
        site_id: "azusa",
        name: "梓喵",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: true,
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                ),
            ],
            ..Default::default()
        }),
    }
}

fn btschool() -> SiteConfig {
    SiteConfig {
        site_id: "btschool",
        name: "学校",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_torrentname_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn chdbits() -> SiteConfig {
    SiteConfig {
        site_id: "chdbits",
        name: "彩虹岛",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: true,
            free_deadline: Some(FieldConfig {
                default_value: Some("{% if fields['downloadvolumefactor']==0 %}{{max_time}}{% endif%}"),
                selector: Some("td[class] > span[title]"),
                attribute: Some("title"),
                filters: vec![Filter::DateParse("%Y-%m-%d %H:%M:%S")],
                ..Default::default()
            }),
            extra: vec![
                ("tags", tags_field_subtitle_div()),
                ("subject", subject_field_subtitle()),
            ],
            ..Default::default()
        }),
    }
}

fn hddolby() -> SiteConfig {
    SiteConfig {
        site_id: "hddolby",
        name: "高清杜比",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn hdhome() -> SiteConfig {
    SiteConfig {
        site_id: "HDHome",
        name: "家园",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: true,
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                ),
            ],
            ..Default::default()
        }),
    }
}

fn hhan() -> SiteConfig {
    let mut query = nexus_search_query();
    query.push(("search-mode", "0"));

    SiteConfig {
        site_id: "hhan",
        name: "憨憨",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query,
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn keepfrds() -> SiteConfig {
    SiteConfig {
        site_id: "keepfrds",
        name: "朋友",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(FieldConfig {
                selector: Some(r#"a[title][href^="details.php?id="]"#),
                attribute: Some("title"),
                ..Default::default()
            }),
            has_imdb: false,
            ..Default::default()
        }),
    }
}

fn ourbits() -> SiteConfig {
    SiteConfig {
        site_id: "ourbits",
        name: "我堡",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: false,
            category_qs_only: true,
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                ),
            ],
            ..Default::default()
        }),
    }
}

fn pterclub() -> SiteConfig {
    SiteConfig {
        site_id: "pterclub",
        name: "猫站",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_tags_subject()),
            has_imdb: false,
            category_qs_only: true,
            extra: vec![
                ("tags", FieldConfig::selector("td.embedded > span.optiontag")),
                (
                    "subject",
                    FieldConfig::selector("td:nth-child(2) > table > tr > td.embedded > span:last-child"),
                ),
            ],
            ..Default::default()
        }),
    }
}

fn ptsbao() -> SiteConfig {
    SiteConfig {
        site_id: "ptsbao",
        name: "烧包乐园",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: "table.torrents > tbody > tr:has(table)",
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_torrentname_td()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn putao() -> SiteConfig {
    SiteConfig {
        site_id: "putao",
        name: "葡萄",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_torrentname_contents()),
            has_imdb: false,
            category_qs_only: true,
            ..Default::default()
        }),
    }
}

fn springsunday() -> SiteConfig {
    let mut query = nexus_search_query();
    query.push(("pick", "0"));

    SiteConfig {
        site_id: "ssd",
        name: "不可说",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query,
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn ultrahd() -> SiteConfig {
    SiteConfig {
        site_id: "ultrahd",
        name: "UltraHD",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn exoticaz() -> SiteConfig {
    SiteConfig {
        site_id: "exoticaz",
        name: "ExoticaZ",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn filelist() -> SiteConfig {
    SiteConfig {
        site_id: "filelist",
        name: "FileList",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "browse.php",
            query: vec![
                ("search", "{{query.keyword}}"),
                ("cat", "0"),
                ("searchin", "0"),
                ("sort", "0"),
            ],
        }),
        adult_search: None,
        row_selector: "div.torrentrow",
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn iptorrents() -> SiteConfig {
    SiteConfig {
        site_id: "iptorrents",
        name: "IPTorrents",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "t",
            query: vec![("q", "{{query.keyword}}")],
        }),
        adult_search: None,
        row_selector: "table#torrents > tbody > tr",
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn hares() -> SiteConfig {
    SiteConfig {
        site_id: "hares",
        name: "白兔",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: "table.torrents > tr",
        fields: nexus_defaults(NexusOverrides {
            description: Some(FieldConfig::selector(
                "div.layui-torrents-Subject > div.left > p.layui-elip.layui-torrents-descr-width",
            )),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn hdatmos() -> SiteConfig {
    SiteConfig {
        site_id: "hdatmos",
        name: "阿童木",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: true,
            ..Default::default()
        }),
    }
}

fn agsv() -> SiteConfig {
    SiteConfig {
        site_id: "agsv",
        name: "末日",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query: nexus_search_query(),
        }),
        adult_search: None,
        row_selector: NEXUS_ROW_SELECTOR,
        fields: nexus_defaults(NexusOverrides {
            description: Some(desc_embedded_contents()),
            has_imdb: false,
            category_qs_only: true,
            ..Default::default()
        }),
    }
}

fn tjupt() -> SiteConfig {
    let mut query = nexus_search_query();
    query.push(("picktype", "0"));

    SiteConfig {
        site_id: "tjupt",
        name: "北洋园",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "torrents.php",
            query,
        }),
        adult_search: None,
        row_selector: "table.torrents > tr:has(table.torrentname)",
        fields: nexus_defaults(NexusOverrides {
            description: Some(FieldConfig::selector("td:nth-child(2)")),
            has_imdb: true,
            category_qs_only: true,
            ..Default::default()
        }),
    }
}

fn totheglory() -> SiteConfig {
    SiteConfig {
        site_id: "ttg",
        name: "听听歌",
        site_type: SiteType::NexusPhp,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "browse.php",
            query: vec![
                ("search_field", "{{query.keyword}}"),
                ("page", "{{ query.page }}"),
                ("c", "M"),
            ],
        }),
        adult_search: None,
        row_selector: "table#torrent_table > tr[id]",
        fields: FieldMap {
            id: FieldConfig {
                selector: Some("div.name_left > a"),
                attribute: Some("href"),
                filters: vec![Filter::ReSearch(r"\d+", 0)],
                ..Default::default()
            },
            title_default: FieldConfig {
                selector: Some("div.name_left > a > b"),
                ..Default::default()
            },
            title_optional: FieldConfig::default(),
            title: FieldConfig::text_tpl("{{ fields['title_default'] }}"),
            description: Some(FieldConfig::text_tpl(
                "{% if fields['description_free_forever'] %}{{ fields['description_free_forever'] }}{% else %}{{ fields['description_normal'] }}{% endif %}",
            )),
            category: Some(FieldConfig {
                selector: Some("tr[id] td:nth-child(1) > a > img"),
                attribute: Some("alt"),
                ..Default::default()
            }),
            details: Some(FieldConfig {
                selector: Some("div.name_left > a"),
                attribute: Some("href"),
                ..Default::default()
            }),
            download: FieldConfig {
                selector: Some("a.dl_a"),
                attribute: Some("href"),
                ..Default::default()
            },
            date_elapsed: None,
            date_added: None,
            date: Some(FieldConfig {
                selector: Some("td:nth-child(5)"),
                filters: vec![Filter::DateParse("%Y-%m-%d%H:%M:%S")],
                ..Default::default()
            }),
            size: FieldConfig::selector("td:nth-child(7)"),
            seeders: FieldConfig {
                selector: Some("td:nth-child(9)"),
                filters: vec![Filter::Split("/", 0)],
                ..Default::default()
            },
            leechers: FieldConfig {
                selector: Some("td:nth-child(9)"),
                filters: vec![Filter::Split("/", 1), Filter::Replace("\n", "")],
                ..Default::default()
            },
            grabs: Some(FieldConfig {
                selector: Some("td:nth-child(8)"),
                filters: vec![Filter::Replace("次", "")],
                ..Default::default()
            }),
            downloadvolumefactor: FieldConfig {
                case: vec![
                    (r#"img[alt="free"]"#, "0"),
                    (r#"img[alt="50%"]"#, "0.5"),
                    (r#"img[alt="30%"]"#, "0.3"),
                    ("*", "1"),
                ],
                ..Default::default()
            },
            uploadvolumefactor: FieldConfig {
                case: vec![(r#"img[alt="200%"]"#, "2"), ("*", "1")],
                ..Default::default()
            },
            free_deadline: Some(FieldConfig {
                default_value: Some("{% if fields['downloadvolumefactor']==0 %}{{max_time}}{% endif%}"),
                selector: Some("span[onclick]"),
                attribute: Some("onclick"),
                filters: vec![
                    Filter::ReSearch(r"\d+年\d+月\d+日\d+点\d+分", 0),
                    Filter::DateParse("%Y年%m月%d日%H点%M分"),
                ],
                ..Default::default()
            }),
            imdbid: Some(FieldConfig {
                selector: Some("span.imdb_rate > a"),
                attribute: Some("href"),
                filters: vec![Filter::ReSearch(r"tt\d+", 0)],
                ..Default::default()
            }),
            extra: vec![
                (
                    "description_free_forever",
                    FieldConfig::selector("div.name_left > a > b > font > span"),
                ),
                (
                    "description_normal",
                    FieldConfig::selector("div.name_left > a > b > span"),
                ),
            ],
        },
    }
}

// ============================================================
// API-based sites
// ============================================================

fn mteam() -> SiteConfig {
    SiteConfig {
        site_id: "m-team",
        name: "馒头",
        site_type: SiteType::Api,
        search: SearchConfig::Api(ApiSearchConfig {
            path: "/api/torrent/search",
            method: "POST",
            headers: vec![("x-api-key", "{{api_key}}")],
            body_template: r#"{"keyword":"{{keyword}}","pageNumber":1,"pageSize":100}"#,
            data_path: Some("data"),
            list_path: Some("data"),
            fields: ApiFieldMap {
                id: "id",
                title: "name",
                subtitle: Some("smallDescr"),
                size: "size",
                seeders: "status.seeders",
                leechers: "status.leechers",
                grabs: Some("status.timesCompleted"),
                category: Some("category"),
                upload_time: Some("createdDate"),
                detail_url: Some("id"),
                poster_url: Some("imageList.0"),
                imdb_url: Some("imdb"),
                imdb_rating: Some("imdbRating"),
                douban_url: Some("douban"),
                douban_rating: Some("doubanRating"),
                discount: Some("status.discount"),
                discount_end_time: Some("status.discountEndTime"),
                video_codec: Some("videoCodec"),
                audio_codec: Some("audioCodec"),
                resolution: Some("standard"),
                source: Some("source"),
            },
        }),
        adult_search: Some(SearchConfig::Api(ApiSearchConfig {
            path: "/api/torrent/search",
            method: "POST",
            headers: vec![("x-api-key", "{{api_key}}")],
            body_template: r#"{"keyword":"{{keyword}}","mode":"adult","categories":[410,429,424,430,426,437,431,432,436,425,433,411,412,413],"pageNumber":1,"pageSize":100}"#,
            data_path: Some("data"),
            list_path: Some("data"),
            fields: ApiFieldMap {
                id: "id",
                title: "name",
                subtitle: Some("smallDescr"),
                size: "size",
                seeders: "status.seeders",
                leechers: "status.leechers",
                grabs: Some("status.timesCompleted"),
                category: Some("category"),
                upload_time: Some("createdDate"),
                detail_url: Some("id"),
                poster_url: Some("imageList.0"),
                imdb_url: Some("imdb"),
                imdb_rating: Some("imdbRating"),
                douban_url: Some("douban"),
                douban_rating: Some("doubanRating"),
                discount: Some("status.discount"),
                discount_end_time: Some("status.discountEndTime"),
                video_codec: Some("videoCodec"),
                audio_codec: Some("audioCodec"),
                resolution: Some("standard"),
                source: Some("source"),
            },
        })),
        row_selector: "",
        fields: FieldMap {
            id: FieldConfig::default(),
            title_default: FieldConfig::default(),
            title_optional: FieldConfig::default(),
            title: FieldConfig::default(),
            description: None,
            category: None,
            details: None,
            download: FieldConfig::default(),
            date_elapsed: None,
            date_added: None,
            date: None,
            size: FieldConfig::default(),
            seeders: FieldConfig::default(),
            leechers: FieldConfig::default(),
            grabs: None,
            downloadvolumefactor: FieldConfig::default(),
            uploadvolumefactor: FieldConfig::default(),
            free_deadline: None,
            imdbid: None,
            extra: Vec::new(),
        },
    }
}

// ============================================================
// Public sites
// ============================================================

fn acg() -> SiteConfig {
    SiteConfig {
        site_id: "acgrip",
        name: "acg",
        site_type: SiteType::Public,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "/",
            query: vec![("term", "{{query.keyword}}")],
        }),
        adult_search: None,
        row_selector: "table > tbody > tr",
        fields: FieldMap {
            id: FieldConfig {
                selector: Some(r#"a[href^="/t/"]"#),
                attribute: Some("href"),
                filters: vec![Filter::ReSearch(r"\d+", 0)],
                ..Default::default()
            },
            title_default: FieldConfig::selector(r#"a[href^="/t/"]"#),
            title_optional: FieldConfig::default(),
            title: FieldConfig::text_tpl("{{ fields['title_default'] }}"),
            description: None,
            category: Some(FieldConfig::default_val("1")),
            details: Some(FieldConfig {
                selector: Some(r#"a[href^="/t/"]"#),
                attribute: Some("href"),
                ..Default::default()
            }),
            download: FieldConfig {
                selector: Some(r#"a[href$=".torrent"]"#),
                attribute: Some("href"),
                ..Default::default()
            },
            date_elapsed: None,
            date_added: None,
            date: Some(FieldConfig::selector("td:nth-child(1) > div")),
            size: FieldConfig::selector("td:nth-child(4)"),
            seeders: FieldConfig::selector("td:nth-child(5)"),
            leechers: FieldConfig::selector("td:nth-child(6)"),
            grabs: Some(FieldConfig::selector("td:nth-child(7)")),
            downloadvolumefactor: FieldConfig::default_val("0"),
            uploadvolumefactor: FieldConfig::default_val("1"),
            free_deadline: Some(FieldConfig::default_val("9999-12-31 23:59:59")),
            imdbid: None,
            extra: Vec::new(),
        },
    }
}

fn mikanani() -> SiteConfig {
    SiteConfig {
        site_id: "mikanani",
        name: "蜜柑",
        site_type: SiteType::Public,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "/Home/Search",
            query: vec![("searchstr", "{{query.keyword}}")],
        }),
        adult_search: None,
        row_selector: "table.table > tbody > tr.js-search-results-row",
        fields: FieldMap {
            id: FieldConfig {
                selector: Some(r#"a[href^="/Download/"]"#),
                attribute: Some("href"),
                filters: vec![Filter::ReSearch(r"[\w-]+\.torrent", 0)],
                ..Default::default()
            },
            title_default: FieldConfig::selector(r#"a[href^="/Home/Episode/"]"#),
            title_optional: FieldConfig::default(),
            title: FieldConfig::text_tpl("{{ fields['title_default'] }}"),
            description: None,
            category: Some(FieldConfig::default_val("1")),
            details: Some(FieldConfig {
                selector: Some(r#"a[href^="/Home/Episode/"]"#),
                attribute: Some("href"),
                ..Default::default()
            }),
            download: FieldConfig {
                selector: Some(r#"a[href^="/Download/"]"#),
                attribute: Some("href"),
                ..Default::default()
            },
            date_elapsed: None,
            date_added: None,
            date: Some(FieldConfig::selector("td:nth-child(3)")),
            size: FieldConfig::selector("td:nth-child(4)"),
            seeders: FieldConfig::selector("td:nth-child(5)"),
            leechers: FieldConfig::selector("td:nth-child(6)"),
            grabs: Some(FieldConfig::selector("td:nth-child(7)")),
            downloadvolumefactor: FieldConfig::default_val("0"),
            uploadvolumefactor: FieldConfig::default_val("1"),
            free_deadline: Some(FieldConfig::default_val("9999-12-31 23:59:59")),
            imdbid: None,
            extra: Vec::new(),
        },
    }
}

fn sukebei() -> SiteConfig {
    SiteConfig {
        site_id: "sukebei",
        name: "Sukebei",
        site_type: SiteType::Public,
        search: SearchConfig::Html(HtmlSearchConfig {
            path: "/",
            query: vec![("f", "0"), ("c", "0_0"), ("q", "{{query.keyword}}")],
        }),
        adult_search: None,
        row_selector: "div.table-responsive > table > tbody > tr",
        fields: FieldMap {
            id: FieldConfig {
                selector: Some(r#"td:nth-child(2) > a[href^="/view/"]"#),
                attribute: Some("href"),
                filters: vec![Filter::ReSearch(r"\d+", 0)],
                ..Default::default()
            },
            title_default: FieldConfig {
                selector: Some(r#"td:nth-child(2) > a[href^="/view/"]"#),
                filters: vec![
                    Filter::Replace("+++", ""),
                    Filter::Replace("[HD]", ""),
                    Filter::Replace("[FHD]", ""),
                    Filter::Replace("[HD/720p]", ""),
                    Filter::Replace("[HD JAV Uncensored]", ""),
                ],
                ..Default::default()
            },
            title_optional: FieldConfig::default(),
            title: FieldConfig::text_tpl("{{ fields['title_default'] }}"),
            description: Some(FieldConfig {
                selector: Some(r#"td:nth-child(2) > a[href^="/view/"]"#),
                filters: vec![
                    Filter::Replace("+++", ""),
                    Filter::Replace("[HD]", ""),
                    Filter::Replace("[FHD]", ""),
                    Filter::Replace("[HD/720p]", ""),
                    Filter::Replace("[HD JAV Uncensored]", ""),
                ],
                ..Default::default()
            }),
            category: Some(FieldConfig::default_val("1")),
            details: Some(FieldConfig {
                selector: Some(r#"td:nth-child(2) > a[href^="/view/"]"#),
                attribute: Some("href"),
                ..Default::default()
            }),
            download: FieldConfig {
                selector: Some(r#"a[href^="/download/"]"#),
                attribute: Some("href"),
                ..Default::default()
            },
            date_elapsed: None,
            date_added: None,
            date: None,
            size: FieldConfig::selector("td:nth-child(4)"),
            seeders: FieldConfig::selector("td:nth-child(6)"),
            leechers: FieldConfig::selector("td:nth-child(7)"),
            grabs: Some(FieldConfig::selector("td:nth-child(8)")),
            downloadvolumefactor: FieldConfig::default_val("0"),
            uploadvolumefactor: FieldConfig::default_val("1"),
            free_deadline: Some(FieldConfig::default_val("9999-12-31 23:59:59")),
            imdbid: None,
            extra: Vec::new(),
        },
    }
}
