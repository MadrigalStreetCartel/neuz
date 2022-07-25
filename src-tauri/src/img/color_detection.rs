use derivative::Derivative;

pub struct ImageAnalyzer {
    image: ImageBuffer,
}

impl ImageAnalyzer {
    #[derive(Derivative)]
        #[derivative(Debug,Default, Clone, Copy)]
    struct AnalyzeZone {
        #[derivative(Default(value = -1))]
        max_x:u32,
        #[derivative(Default(value = -1))]
        max_y:u32,
        #[derivative(Default(value = -1))]
        start_x:u32,
        #[derivative(Default(value = -1))]
        start_y:u32,
    }
    fn AnalyzeZone(max_x: u32, max_y: u32,start_x: u32,start_y: u32) -> User {
        User {
            max_x,
            max_y,
            start_x,
            start_y,
        }
    }

    pub fn find_color<'a>(&self,colors:[([u8; 3],fn(x:u32,y:u32,snd:SyncSender)->None);0] ,cb:fn(point:Point, point_cloud:PointCloud)->None,analyze_zone:Option<AnalyzeZone>) -> PointCloud
    {

        const detect_zone:AnalyzeZone = analyze_zone.unwrap_or_default();
        let (snd, recv) = sync_channel::<'a>(4096);
        self.image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP || y > self.image.height() - IGNORE_AREA_BOTTOM || detect_zone.max_y != -1 &&  y > IGNORE_AREA_TOP + detect_zone.max_y {
                    return;
                }
                for (x, _, px) in row {
                    if px.0[3] != 255 || detect_zone.max_x != -1 && x >= detect_zone.max_x {
                        return;
                    }
                    for color in colors.iter() {
                        if Self::pixel_matches(&px.0, &color.0, 2) {
                            color.1(x,y);
                        }
                    }
                }
            });

            let cloud = {
                let mut cloud = PointCloud::default();
                while let Ok(point) = recv.recv() {
                    cb(point,cloud);
                }
                cloud
            };
            cloud
    }

    pub fn detect_stats_bar(&self, last_hp: Stat, status_bar: StatusBar) -> Option<Stat> {
        let status_bar_config = match status_bar {
            StatusBar::Hp => HP_BAR,
            StatusBar::Mp => MP_BAR,
            StatusBar::Fp => FP_BAR,
            StatusBar::Xp => XP_BAR,
        };


        // Receive points from channel
        let cloud = self.find_color<Point>(
            [(status_bar_config.refs, |x,y,snd| {
                #[allow(clippy::drop_copy)]
                drop(snd.send(Point::new(x, y)));
                continue 'outer;
            }), |point, cloud| cloud.push(point)],
            AnalyzeZone(310,120)
        );

        // Calculate bounds
        let bounds = cloud.to_bounds();

        // Recalculate hp tracking info
        let max_w = bounds.w.max(last_hp.max_w);
        let hp_frac = bounds.w as f32 / max_w as f32;
        let hp_scaled = ((hp_frac * 100_f32) as u32).max(0).min(100);
        let hp = Stat {
            max_w,
            value: hp_scaled,
        };

        Some(hp)
    }

    pub fn identify_mobs(&self) -> Vec<Target> {
        let _timer = Timer::start_new("identify_mobs");

        // Create collections for passive and aggro mobs
        let mut mob_coords_pas: Vec<Point> = Vec::default();
        let mut mob_coords_agg: Vec<Point> = Vec::default();

        // Reference colors
        let ref_color_pas: [u8; 3] = [0xe8, 0xe8, 0x94]; // Passive mobs
        let ref_color_agg: [u8; 3] = [0xd3, 0x0f, 0x0d]; // Aggro mobs

        // Collect pixel clouds
        struct MobPixel(u32, u32, TargetType);
        let (snd, recv) = sync_channel::<MobPixel>(4096);
        self.image
            .enumerate_rows()
            .par_bridge()
            .for_each(move |(y, row)| {
                #[allow(clippy::absurd_extreme_comparisons)] // not always 0 (macOS)
                if y <= IGNORE_AREA_TOP || y > self.image.height() - IGNORE_AREA_BOTTOM {
                    return;
                }
                for (x, _, px) in row {
                    if px.0[3] != 255 || y > self.image.height() - IGNORE_AREA_BOTTOM {
                        return;
                    }
                    if Self::pixel_matches(&px.0, &ref_color_pas, 2) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Passive))));
                    } else if Self::pixel_matches(&px.0, &ref_color_agg, 8) {
                        drop(snd.send(MobPixel(x, y, TargetType::Mob(MobType::Aggressive))));
                    }
                }
            });
        while let Ok(px) = recv.recv() {
            match px.2 {
                TargetType::Mob(MobType::Passive) => mob_coords_pas.push(Point::new(px.0, px.1)),
                TargetType::Mob(MobType::Aggressive) => mob_coords_agg.push(Point::new(px.0, px.1)),
                _ => unreachable!(),
            }
        }

        // Categorize mobs
        let mobs_pas = Self::merge_cloud_into_mobs(
            &PointCloud::new(mob_coords_pas),
            TargetType::Mob(MobType::Passive),
            false,
        );
        let mobs_agg = Self::merge_cloud_into_mobs(
            &PointCloud::new(mob_coords_agg),
            TargetType::Mob(MobType::Aggressive),
            false,
        );

        // Return all mobs
        Vec::from_iter(mobs_agg.into_iter().chain(mobs_pas.into_iter()))
    }


}
