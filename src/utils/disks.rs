use sysinfo::Disks;

pub fn get_mount_points() -> Vec<String> {
    let disks = Disks::new_with_refreshed_list();
    let mut mount_points: Vec<String> = Vec::new();
    for disk in &disks {
        let mount_point_str = disk.mount_point().to_str().unwrap_or_default().to_string();
        if mount_point_str.starts_with("/media") || mount_point_str.starts_with("/run/media") {
            mount_points.push(mount_point_str);
        }
    }
    mount_points
}
