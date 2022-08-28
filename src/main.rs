use chrono::{Local, Duration, Datelike, Timelike};
use structopt::StructOpt;

#[derive(structopt::StructOpt)]
struct Opts {
	/// The URL of the schedule endpoint on the upstream ETHSBell instance.
	#[structopt(short="u", long, default_value="https://ethsbell.app/api/v1/schedule")]
	pub upstream_ethsbell: String,
	/// The time zone offset as a number of hours. Only set this if your system's time zone does not match that of the upstream.
	#[structopt(short, long, default_value="0")]
	pub timezone_offset: f32,
	/// One Discord webhook url
	#[structopt(short, long)]
	pub discord_urls: Vec<String>,
	/// A file containing a list of Discord webhook urls
	#[structopt(short="i", long)]
	pub discord_url_file: Vec<String>,
	/// Number of days into the future to generate events for
	#[structopt(short="n", long, default_value="7")]
	pub days_into_future: i64,
}

#[tokio::main]
async fn main() {
	let opts = Opts::from_args();
	let schedule: ethsbell_rewrite::schedule::Schedule = {
		let response = reqwest::get(opts.upstream_ethsbell).await.expect("Failed to reach upstream").text().await.expect("Got non-text response");
		serde_json::from_str(&response).expect("Invalid response")
	};
	let urls: Vec<String> = opts.discord_urls
		.iter()
		.cloned()
		.chain(
			opts.discord_url_file.iter()
				.flat_map(std::fs::read_to_string)
				.flat_map(|file| file.lines().into_iter().map(|v| v.to_string()).collect::<Vec<String>>())
				.filter(|s| !s.is_empty())
		).collect();
	eprintln!("Got schedule from upstream, generating events for today through {} days into the future", opts.days_into_future);
	let now = Local::now();
	for mut day in (0..opts.days_into_future).into_iter().map(|n| now + Duration::days(n)) {
		let mut daily_schedule = schedule.on_date(day.date_naive()).0;
		daily_schedule.periods = daily_schedule.periods.into_iter().map(|period| period.populate(day)).map(|mut v| {
			v.start += Duration::minutes((opts.timezone_offset * 60.0) as i64);
			v.end += Duration::minutes((opts.timezone_offset * 60.0) as i64);
			v
		}).collect();
		day += Duration::minutes((opts.timezone_offset * 60.0) as i64);
		for period in daily_schedule.periods {
			let alerts = [
				(format!("{} begins in 5 minutes", period.friendly_name), period.start - Duration::minutes(5)),
				(format!("{} begins now!", period.friendly_name), period.start),
				(format!("{} ends in 5 minutes", period.friendly_name), period.end - Duration::minutes(5)),
				(format!("{} ends now!", period.friendly_name), period.end),
			];
			for alert in alerts {
				for url in urls.iter() {
					println!("{minutes} {hours} {month_day} {month} {weekday} {command}",
						minutes = alert.1.minute(),
						hours = alert.1.hour(),
						month_day = day.date_naive().day(),
						month = day.month(),
						weekday = day.weekday().num_days_from_sunday(),
						command = format_args!(
							"curl -i -H \"Accept: application/json\" -H \"Content-Type: application/json\" -X POST --data \"{{\\\"content\\\": \\\"{}\\\"}}\" {}",
							alert.0,
							url
						)
					);
				}
			}
		}
	}
}
