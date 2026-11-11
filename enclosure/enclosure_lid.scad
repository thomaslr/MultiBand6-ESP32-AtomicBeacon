// SPDX-License-Identifier: CERN-OHL-S-2.0
// Copyright (c) 2026 Robert Longbottom (@thomaslr)
// Commercial licensing available — see LICENSE for details.

// =============================================================================
// MultiBand 6 ESP32 Beacon — Parametric Enclosure Lid
// Snap-Fit Top Cover with Inductor Dome, LED Light Pipe, and Button Actuator
// =============================================================================

$fn = 48;

// --- Dimensions (mm) ---
pcb_x = 58.0;
pcb_y = 36.0;
clearance = 0.5;

wall_t = 2.0;
lid_top_t = 2.0;

inner_x = pcb_x + 2 * clearance; // 59.0
inner_y = pcb_y + 2 * clearance; // 37.0

outer_x = inner_x + 2 * wall_t; // 63.0
outer_y = inner_y + 2 * wall_t; // 41.0
corner_r = 3.0;

// Inductor L1 position relative to PCB origin: (45.0, 25.0)
ind_x = wall_t + clearance + 45.0;
ind_y = wall_t + clearance + 25.0;
ind_od = 19.0;
ind_id = 17.0;
ind_dome_h = 14.0;

// LED Light Pipe position (SuperMini LED on GPIO 8)
led_x = wall_t + clearance + 16.12;
led_y = wall_t + clearance + 13.62;

// Tactile Button SW1 position: (28.0, 32.5)
btn_x = wall_t + clearance + 28.0;
btn_y = wall_t + clearance + 32.5;

// Screw Terminal Block J1: (51.0, 6.0)
term_x = wall_t + clearance + 51.0;
term_y = wall_t + clearance + 6.0;

// Helper: Rounded Box
module rounded_box(x, y, z, r) {
    hull() {
        translate([r, r, 0]) cylinder(r=r, h=z);
        translate([x - r, r, 0]) cylinder(r=r, h=z);
        translate([r, y - r, 0]) cylinder(r=r, h=z);
        translate([x - r, y - r, 0]) cylinder(r=r, h=z);
    }
}

module enclosure_lid() {
    difference() {
        union() {
            // Top Plate Cap
            rounded_box(outer_x, outer_y, lid_top_t, corner_r);
            
            // Inner Mating Lip (inserts into base)
            translate([wall_t + 0.15, wall_t + 0.15, -2.5])
                rounded_box(inner_x - 0.3, inner_y - 0.3, 2.5, corner_r - 0.5);
                
            // Inductor Tower Dome (Cylinder with rounded top)
            translate([ind_x, ind_y, 0]) {
                cylinder(d=ind_od, h=ind_dome_h);
                translate([0, 0, ind_dome_h])
                    sphere(d=ind_od);
            }
        }
        
        // Hollow interior for inner lip
        translate([wall_t + 1.2, wall_t + 1.2, -3.0])
            rounded_box(inner_x - 2.4, inner_y - 2.4, 3.5, corner_r - 1.0);
            
        // Hollow Inductor Cavity
        translate([ind_x, ind_y, -3.0]) {
            cylinder(d=ind_id, h=ind_dome_h + 3.0);
            translate([0, 0, ind_dome_h + 3.0])
                sphere(d=ind_id);
        }
        
        // Status LED Light Pipe Pinhole (GPIO 8)
        translate([led_x, led_y, -3.0])
            cylinder(d=1.8, h=lid_top_t + 6.0);
            
        // Button SW1 Access Hole (Manual Sync Pushbutton)
        translate([btn_x, btn_y, -3.0])
            cylinder(d=4.0, h=lid_top_t + 6.0);
            
        // Terminal Block J1 Wire Port
        translate([term_x - 4.0, -1.0, -3.0])
            cube([8.0, wall_t + 3.0, 5.0]);
            
        // Subtle Embossed Title
        translate([12.0, outer_y - 8.0, lid_top_t - 0.4])
            linear_extrude(height = 0.5)
                text("MultiBand 6", size = 3.6, font = "Helvetica:style=Bold");
    }
}

enclosure_lid();
