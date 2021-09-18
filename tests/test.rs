use tracing_oslog::OsLogger;
use tracing_subscriber::layer::SubscriberExt;

#[tracing::instrument]
async fn blah() {
	tracing::info!("info");
	tracing::info!("error");
	tracing::warn!("warn");
	tracing::error!("hi");
	tracing::debug!("hi");
	example::thingy().await;
}

mod example {
	#[tracing::instrument]
	pub async fn thingy() {
		tracing::info!("info");
		tracing::info!("error");
		tracing::warn!("warn");
		tracing::error!("hi");
		tracing::debug!("hi");
	}
}

#[tracing::instrument]
#[tokio::test]
async fn main() {
	let collector = tracing_subscriber::registry().with(OsLogger::new("moe.absolucy.test"));
	tracing::subscriber::set_global_default(collector).expect("a");
	blah().await;
}
