// SPDX-License-Identifier: CERN-OHL-S-2.0
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

// =============================================================================
// MultiBand 6 ESP32 Beacon — Parametric Enclosure Base
// Multi-Mount Support: Screws, Cable Ties, 3M VHB/Command Tape, Attachable Stand
// =============================================================================

$fn = 48;

// --- Dimensions (mm) ---
pcb_x = 58.0;
pcb_y = 36.0;
pcb_z = 1.6;
clearance = 0.5;

wall_t = 2.0;
floor_t = 2.0;

inner_x = pcb_x + 2 * clearance; // 59.0
inner_y = pcb_y + 2 * clearance; // 37.0
box_h = 10.0; // base wall height

outer_x = inner_x + 2 * wall_t; // 63.0
outer_y = inner_y + 2 * wall_t; // 41.0
corner_r = 3.0;

// PCB Standoffs (M2.5 self-tapping)
standoff_h = 3.5;
standoff_od = 5.5;
pilot_id = 2.2;

// Mounting Ear Flanges
ear_w = 10.0;
ear_hole_dia = 3.6; // M3 screw clearance
ear_csink_dia = 6.2;

// Cable Tie Channels
tie_w = 4.5;
tie_d = 2.0;

// Helper: Rounded Box
module rounded_box(x, y, z, r) {
    hull() {
        translate([r, r, 0]) cylinder(r=r, h=z);
        translate([x - r, r, 0]) cylinder(r=r, h=z);
        translate([r, y - r, 0]) cylinder(r=r, h=z);
        translate([x - r, y - r, 0]) cylinder(r=r, h=z);
    }
}

module enclosure_base() {
    difference() {
        union() {
            // Main Base Body
            rounded_box(outer_x, outer_y, box_h + floor_t, corner_r);
            
            // Left Mounting Ear (Screw Mount)
            translate([-ear_w, 0, 0])
                rounded_box(ear_w + corner_r, outer_y, floor_t + 1.0, corner_r);
                
            // Right Mounting Ear (Screw Mount)
            translate([outer_x - corner_r, 0, 0])
                rounded_box(ear_w + corner_r, outer_y, floor_t + 1.0, corner_r);
        }
        
        // Interior Cavity
        translate([wall_t, wall_t, floor_t])
            rounded_box(inner_x, inner_y, box_h + 1, corner_r - 0.5);
            
        // Left Ear Screw Hole & Countersink
        translate([-ear_w / 2, outer_y / 2, -1]) {
            cylinder(d=ear_hole_dia, h=floor_t + 4);
            translate([0, 0, floor_t + 1 - 1.2])
                cylinder(d1=ear_hole_dia, d2=ear_csink_dia, h=2.5);
        }
        
        // Right Ear Screw Hole & Countersink
        translate([outer_x + ear_w / 2, outer_y / 2, -1]) {
            cylinder(d=ear_hole_dia, h=floor_t + 4);
            translate([0, 0, floor_t + 1 - 1.2])
                cylinder(d1=ear_hole_dia, d2=ear_csink_dia, h=2.5);
        }
        
        // USB-C Cutout (Centered on SuperMini, PCB X = 16.12 + wall_t + clearance)
        translate([wall_t + clearance + 16.12 - 6.0, -1, floor_t + standoff_h + pcb_z])
            cube([12.0, wall_t + 2, 7.0]);
            
        // Recessed Cable-Tie Slot 1 (Horizontal)
        translate([outer_x / 2 - tie_w / 2, -1, -0.1])
            cube([tie_w, outer_y + 2, tie_d]);
            
        // Recessed Cable-Tie Slot 2 (Vertical)
        translate([-1, outer_y / 2 - tie_w / 2, -0.1])
            cube([outer_x + 2, tie_w, tie_d]);
            
        // 3M Tape Recessed Bed on Underside (Flat adhesion zone)
        translate([outer_x / 2 - 20, outer_y / 2 - 12, -0.1])
            rounded_box(40, 24, 0.4, 2.0);
            
        // Stand Coupling Slots (Side rail notches for clip-on desk stand)
        translate([outer_x / 2 - 12, -1, 3.0])
            cube([24, wall_t * 0.75, 4.0]);
        translate([outer_x / 2 - 12, outer_y - wall_t * 0.75 + 1, 3.0])
            cube([24, wall_t * 0.75, 4.0]);
    }
    
    // Internal PCB Mounting Standoffs
    // Relative to PCB: (3.5, 3.5), (54.5, 3.5), (3.5, 32.5), (54.5, 32.5)
    pcb_holes = [
        [3.5, 3.5],
        [54.5, 3.5],
        [3.5, 32.5],
        [54.5, 32.5]
    ];
    
    for (pos = pcb_holes) {
        translate([wall_t + clearance + pos[0], wall_t + clearance + pos[1], floor_t]) {
            difference() {
                cylinder(d=standoff_od, h=standoff_h);
                translate([0, 0, -0.5])
                    cylinder(d=pilot_id, h=standoff_h + 1);
            }
        }
    }
}

enclosure_base();
