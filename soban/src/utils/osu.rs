use std::sync::Arc;

use eyre::{Report, Result};

use rosu_pp::{BeatmapExt, DifficultyAttributes, PerformanceAttributes};
use rosu_v2::{
    prelude::{GameMods, OsuError, Score},
    request::UserId,
};

use crate::{utils::datetime::RelativeTime, CommandOrigin, Context};

use super::beatmap::get_beatmap;

struct CalculatedScore {
    cs: f32,
    ar: f64,
    od: f64,
    stars: f64,
    fc_pp: f64,
}

pub struct RecentArgs {
    pub user: UserId,
    pub idx: Option<u32>,
    pub include_fails: bool,
}

pub async fn handle_osu(
    ctx: Arc<Context>,
    origin: CommandOrigin<'_>,
    user_id: UserId,
) -> Result<()> {
    match format_user(ctx, user_id).await {
        Ok(response) => origin.send(&response).await?,
        Err(err) => {
            if let Some(OsuError::NotFound) = err.downcast_ref::<OsuError>() {
                origin.send("couldn't find user").await?;
            } else {
                origin.send("couldn't reach osu!api").await?;
                return Err(err);
            }
        }
    }

    Ok(())
}

pub async fn handle_recent(
    ctx: Arc<Context>,
    origin: CommandOrigin<'_>,
    args: RecentArgs,
) -> Result<()> {
    match get_recent(ctx, args).await {
        Ok(response) => origin.send(&response).await?,
        Err(err) => {
            if let Some(OsuError::NotFound) = err.downcast_ref::<OsuError>() {
                origin.send("couldn't find user").await?;
            } else {
                origin.send("couldn't reach osu!api").await?;
                return Err(err);
            }
        }
    }

    Ok(())
}

async fn get_recent(ctx: Arc<Context>, args: RecentArgs) -> Result<String> {
    let offset = args.idx.unwrap_or(1).saturating_sub(1) as usize;

    let scores = ctx
        .osu
        .user_scores(args.user)
        .recent()
        .include_fails(args.include_fails)
        .offset(offset)
        .limit(1)
        .await?;

    if scores.is_empty() {
        return Ok("no recent scores found".to_owned());
    }

    let score = &scores[0];
    let response = format_score(score).await?;

    Ok(response)
}

async fn format_user(ctx: Arc<Context>, user_id: UserId) -> Result<String> {
    let osu_user = ctx.osu.user(user_id).await?;
    let osu_user_stats = osu_user.statistics.as_ref().expect("missing user stats");
    let rank = osu_user_stats
        .global_rank
        .map_or("-".to_owned(), |r| r.to_string());
    let country_rank = osu_user_stats
        .country_rank
        .map_or("-".to_owned(), |r| r.to_string());

    let response = format!(
        "{username} - {pp}pp (#{rank}) ({country_code}#{country_rank})\nRanked Score: {ranked_score}",
        username = osu_user.username,
        pp = osu_user_stats.pp,
        country_code = osu_user.country_code,
        ranked_score = osu_user_stats.ranked_score,
    );

    Ok(response)
}

async fn format_score(score: &Score) -> Result<String> {
    let calc = calculate_score(score).await?;

    let map_stats = format!(
        "CS{cs:.2} AR{ar:.2} OD{od:.2} ★{stars:.2}",
        cs = calc.cs,
        ar = calc.ar,
        od = calc.od,
        stars = calc.stars
    );

    let pp = match score.pp {
        Some(pp) => format!(" {pp:.2}pp"),
        None => "".to_owned(),
    };

    let (fc_pp, fc_or_misses) = match score.perfect {
        false => (
            format!(" >> {:.2}pp if FC", calc.fc_pp),
            format!("{}m ", score.statistics.count_miss),
        ),
        true => ("".to_owned(), "FC ".to_owned()),
    };

    Ok(format!(
        "https://osu.ppy.sh/b/{id} {grade} +{mods} {acc:.2}% {fc_or_misses}{map_stats}{pp}{fc_pp} - {date}",
        grade = score.grade,
        id = score.map_id,
        mods = score.mods,
        acc = score.accuracy,
        date = score.ended_at.to_relative(),
    ))
}

pub fn parse_user_id(input: &str) -> Option<UserId> {
    if input.is_empty() {
        return None;
    }

    let id = input
        .split_whitespace()
        .next()
        .and_then(|arg| arg.parse::<u32>().ok())
        .map_or_else(|| UserId::Name(input.into()), UserId::Id);

    Some(id)
}

pub async fn require_user_id(origin: CommandOrigin<'_>) -> Result<()> {
    origin.send("missing username").await?;

    Ok(())
}

async fn calculate_score(score: &Score) -> Result<CalculatedScore> {
    let map = get_beatmap(score.map_id).await?;
    let attr = map.stars().mods(score.mods.bits()).calculate();
    let pp_res = map
        .pp()
        .attributes(attr)
        .mods(score.mods.bits())
        .n100(score.statistics.count_100 as usize)
        .n50(score.statistics.count_50 as usize)
        .calculate();

    let cs = if score.mods.contains(GameMods::HardRock) {
        map.cs * 1.3
    } else if score.mods.contains(GameMods::Easy) {
        map.cs * 0.5
    } else {
        map.cs
    };

    match (pp_res.difficulty_attributes(), pp_res) {
        (DifficultyAttributes::Osu(map_attr), PerformanceAttributes::Osu(pp)) => {
            Ok(CalculatedScore {
                cs,
                ar: map_attr.ar,
                od: map_attr.od,
                stars: map_attr.stars,
                fc_pp: pp.pp,
            })
        }
        _ => return Err(Report::msg("not an osu map")),
    }
}
