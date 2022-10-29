pub struct SupportBehavior<'a> {
    rng: rand::rngs::ThreadRng,
    logger: &'a Logger,
    platform: &'a PlatformAccessor<'a>,
    movement: &'a MovementAccessor,
    slots_usage_last_time: [[Option<Instant>; 10]; 9],
}

impl<'a> Behavior<'a> for SupportBehavior<'a> {
    fn new(
        platform: &'a PlatformAccessor<'a>,
        logger: &'a Logger,
        movement: &'a MovementAccessor,
    ) -> Self {
        Self {
            rng: rand::thread_rng(),
            logger,
            platform,
            movement,
            slots_usage_last_time: [[None; 10]; 9],
        }
    }

    fn start(&mut self, _config: &BotConfig) {}
    fn update(&mut self, _config: &BotConfig) {}
    fn stop(&mut self, _config: &BotConfig) {}

    fn run_iteration(
        &mut self,
        frontend_info: &mut FrontendInfo,
        config: &BotConfig,
        image: &mut ImageAnalyzer,
    ) {
        let config = config.support_config();

       /*  // Update all needed timestamps
        self.update_timestamps(config);

        // Check whether something should be restored
        self.check_restorations(config, image);

        // Use buffs Yiha
        self.check_buffs(config); */
    }


}

impl<'a> SupportBehavior<'_> {


}
