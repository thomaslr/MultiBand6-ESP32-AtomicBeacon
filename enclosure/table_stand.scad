// =============================================================================
// MultiBand 6 ESP32 Beacon — Attachable Nightstand / Table Stand
// 45-Degree Angled Cradle for Horizontal Desks, Nightstands, and Tables
// =============================================================================

$fn = 48;

// --- Dimensions (mm) ---
cradle_w = 46.0;   // width matching enclosure base
cradle_depth = 42.0; // depth to hold enclosure
tilt_angle = 45.0; // 45-degree angle pointing antenna upward

base_plate_w = 60.0;
base_plate_l = 68.0;
base_plate_h = 3.5;

wall_t = 3.0;
lip_h = 7.0;

module rounded_rect(w, l, h, r) {
    hull() {
        translate([r, r, 0]) cylinder(r=r, h=h);
        translate([w - r, r, 0]) cylinder(r=r, h=h);
        translate([r, l - r, 0]) cylinder(r=r, h=h);
        translate([w - r, l - r, 0]) cylinder(r=r, h=h);
    }
}

module table_stand() {
    difference() {
        union() {
            // Flat Stable Base Foot
            rounded_rect(base_plate_w, base_plate_l, base_plate_h, 4.0);
            
            // Angled Support Wedges
            translate([base_plate_w / 2 - cradle_w / 2, 8.0, 0]) {
                // Left Wedge
                hull() {
                    cube([wall_t, 50.0, base_plate_h]);
                    translate([0, 38.0, 28.0])
                        cube([wall_t, 10.0, 2.0]);
                }
                
                // Right Wedge
                translate([cradle_w - wall_t, 0, 0]) {
                    hull() {
                        cube([wall_t, 50.0, base_plate_h]);
                        translate([0, 38.0, 28.0])
                            cube([wall_t, 10.0, 2.0]);
                    }
                }
                
                // Center Angled Bed Backing
                translate([0, 16.0, 0]) {
                    rotate([tilt_angle, 0, 0]) {
                        difference() {
                            cube([cradle_w, cradle_depth, wall_t]);
                            
                            // Center Weight Reduction Cutout
                            translate([cradle_w / 2 - 12.0, 8.0, -1])
                                rounded_rect(24.0, 24.0, wall_t + 2, 3.0);
                        }
                    }
                }
                
                // Bottom Retention Shelf / Lip (Holds base from sliding down)
                translate([0, 16.0, 0]) {
                    rotate([tilt_angle, 0, 0]) {
                        translate([0, 0, wall_t])
                            cube([cradle_w, lip_h, wall_t]);
                    }
                }
            }
        }
        
        // 4x Anti-Slip Rubber Foot Recesses (8 mm dia, 1 mm deep) on underside
        foot_r = 4.0;
        translate([8.0, 8.0, -0.1]) cylinder(r=foot_r, h=1.2);
        translate([base_plate_w - 8.0, 8.0, -0.1]) cylinder(r=foot_r, h=1.2);
        translate([8.0, base_plate_l - 8.0, -0.1]) cylinder(r=foot_r, h=1.2);
        translate([base_plate_w - 8.0, base_plate_l - 8.0, -0.1]) cylinder(r=foot_r, h=1.2);
    }
}

table_stand();
