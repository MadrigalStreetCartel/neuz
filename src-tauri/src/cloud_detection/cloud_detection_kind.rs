crate::create_cloud_detection_kinds!(CloudDetectionKind, {
    Hp = ([[346.0, 0.86, 0.75]], Some([0.3, 0.1, 0.2]));

    Mp = ([[214.0, 0.86, 0.75]], Some([0.5, 0.1, 0.2]));

    Fp = ([[119.0, 0.86, 0.65]], Some([0.5, 0.1, 0.2]));

    MobAggressive = ([[0.0, 0.98, 0.70]], Some([0.1, 0.1, 0.2]));

    MobPassive = ([[60.0, 0.36, 1.0]], Some([0.1, 0.05, 0.1]));

    TargetAggressive = ([[354.0, 0.6, 0.9]], Some([0.5, 0.05, 0.1]));

    TargetPassive = ([[226.0, 0.36, 0.8]], Some([0.3, 0.05, 0.1]));
});
