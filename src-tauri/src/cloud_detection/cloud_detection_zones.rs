crate::create_cloud_detection_zones!(CloudDetectionZone, {
    Enemy = Bounds {
        // Monster health panel
        x: 300,
        y: 30,
        w: 550,
        h: 60,
    };
    Player = Bounds {
        // Player health panel aka stats tray
        x: 105,
        y: 30,
        w: 225,
        h: 110,
    };
    Full = Bounds {
        // Full screen not really full cause of status bar
        x: 200,
        y: 100,
        w: 800,
        h: 600,
    };
});
