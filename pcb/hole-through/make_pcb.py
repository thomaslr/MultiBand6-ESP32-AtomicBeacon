#!/usr/bin/env python3
"""
MultiBand 6 Atomic Clock Beacon - ESP32-C3 Carrier (Through-Hole Edition)
Automated KiCad 10 / 9 Project & Manufacturing Generator
100% Through-Hole (THT) Components for easy DIY hand-soldering.
"""

import os
import uuid
import shutil
import subprocess
import zipfile

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_NAME = "MultiBand6_Beacon_THT"

def gen_uuid():
    return str(uuid.uuid4())

# -------------------------------------------------------------------------
# 1. GENERATE KICAD PROJECT (.kicad_pro)
# -------------------------------------------------------------------------
def create_project():
    pro_path = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}.kicad_pro")
    
    # Read settings from main project or fallback to standard rules
    main_pro = os.path.join(os.path.dirname(SCRIPT_DIR), "MultiBand6_Beacon.kicad_pro")
    if os.path.exists(main_pro):
        with open(main_pro, "r", encoding="utf-8") as f:
            content = f.read()
        # Ensure project-specific ignore settings
        replacements = {
            '"silk_over_copper": "warning"': '"silk_over_copper": "ignore"',
            '"silk_overlap": "warning"': '"silk_overlap": "ignore"',
            '"solder_mask_bridge": "error"': '"solder_mask_bridge": "ignore"',
            '"courtyards_overlap": "error"': '"courtyards_overlap": "ignore"',
            '"pth_inside_courtyard": "error"': '"pth_inside_courtyard": "ignore"',
            '"npth_inside_courtyard": "error"': '"npth_inside_courtyard": "ignore"',
            '"lib_footprint_issues": "warning"': '"lib_footprint_issues": "ignore"',
            '"lib_footprint_mismatch": "warning"': '"lib_footprint_mismatch": "ignore"',
        }
        for old, new in replacements.items():
            content = content.replace(old, new)
    else:
        content = """{
  "board": {
    "design_settings": {
      "rules": {
        "max_error": 0.005,
        "min_clearance": 0.2,
        "min_copper_edge_clearance": 0.3,
        "min_hole_clearance": 0.25,
        "min_hole_to_hole": 0.25,
        "min_microvia_diameter": 0.2,
        "min_microvia_drill": 0.1,
        "min_silk_clearance": 0.0,
        "min_text_height": 0.6,
        "min_through_hole_diameter": 0.3,
        "min_track_width": 0.2,
        "min_via_annular_width": 0.1,
        "min_via_diameter": 0.6,
        "solder_mask_clearance": 0.05,
        "solder_mask_min_width": 0.1
      },
      "rule_severities": {
        "courtyards_overlap": "ignore",
        "lib_footprint_issues": "ignore",
        "lib_footprint_mismatch": "ignore",
        "missing_courtyard": "ignore",
        "npth_inside_courtyard": "ignore",
        "pth_inside_courtyard": "ignore",
        "silk_over_copper": "ignore",
        "silk_overlap": "ignore",
        "solder_mask_bridge": "ignore"
      }
    }
  }
}
"""
    with open(pro_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Created project: {pro_path}")

# -------------------------------------------------------------------------
# 2. GENERATE KICAD SCHEMATIC (.kicad_sch)
# -------------------------------------------------------------------------
def create_schematic():
    sch_path = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}.kicad_sch")
    sch_uuid = gen_uuid()
    content = f"""(kicad_sch
	(version 20260206)
	(generator "eeschema")
	(generator_version "10.0")
	(uuid "{sch_uuid}")
	(paper "A4")
	(title_block
		(title "MultiBand 6 Atomic Clock Beacon - ESP32-C3 Carrier (Through-Hole Edition)")
		(date "2026-10-14")
		(rev "v1.0")
		(company "MultiBand6-ESP32-AtomicBeacon (@thomaslr)")
		(comment 1 "100% Through-Hole (THT) Hand-Solderable Design")
		(comment 2 "C1815 / 2N2222 TO-92 NPN + 1N5819 DO-41 Diode + 1/4W Axial Resistors")
	)
	(lib_symbols)
)
"""
    with open(sch_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Created schematic: {sch_path}")

# -------------------------------------------------------------------------
# 3. GENERATE KICAD PCB LAYOUT (.kicad_pcb)
# -------------------------------------------------------------------------
def create_pcb():
    pcb_path = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}.kicad_pcb")

    # Board Dimensions: 70.0 mm x 46.0 mm with 3.0 mm rounded corners
    # Aux Axis Origin anchored at bottom-left corner: (0, 46.0)
    pcb_content = f"""(kicad_pcb
	(version 20260206)
	(generator "pcbnew")
	(generator_version "10.0")
	(general
		(thickness 1.6)
		(legacy_teardrops no)
	)
	(paper "A4")
	(title_block
		(title "MultiBand 6 Atomic Clock Beacon - ESP32-C3 Carrier (Through-Hole Edition)")
		(date "2026-10-14")
		(rev "v1.0")
		(company "MultiBand6-ESP32-AtomicBeacon")
		(comment 1 "100% Through-Hole (THT) Hand-Solderable DIY Design")
		(comment 2 "70 mm x 46 mm Footprint with M3 Corner Mounting Holes")
	)
	(layers
		(0 "F.Cu" signal)
		(31 "B.Cu" signal)
		(32 "B.Adhes" user "B.Adhesive")
		(33 "F.Adhes" user "F.Adhesive")
		(34 "B.Paste" user)
		(35 "F.Paste" user)
		(36 "B.SilkS" user "B.Silkscreen")
		(37 "F.SilkS" user "F.Silkscreen")
		(38 "B.Mask" user)
		(39 "F.Mask" user)
		(40 "Dwgs.User" user "User.Drawings")
		(41 "Cmts.User" user "User.Comments")
		(42 "Eco1.User" user "User.Eco1")
		(43 "Eco2.User" user "User.Eco2")
		(44 "Edge.Cuts" user)
		(45 "Margin" user)
		(46 "B.CrtYd" user "B.Courtyard")
		(47 "F.CrtYd" user "F.Courtyard")
		(48 "B.Fab" user)
		(49 "F.Fab" user)
	)
	(setup
		(pad_to_mask_clearance 0.05)
		(solder_mask_min_width 0.1)
		(pad_to_paste_clearance 0)
		(aux_axis_origin 0 46)
	)
	(net 0 "")
	(net 1 "GND")
	(net 2 "+5V")
	(net 3 "GPIO1_CARRIER")
	(net 4 "NET_BASE")
	(net 5 "NET_TANK_SW")
	(net 6 "NET_TANK_HI")
	(net 7 "+3V3")
	(net 10 "GPIO0")
	(net 11 "GPIO2")
	(net 12 "GPIO3")
	(net 13 "GPIO4")
	(net 14 "GPIO5")
	(net 15 "GPIO6")
	(net 16 "GPIO7")
	(net 17 "GPIO8")
	(net 18 "GPIO9")
	(net 19 "GPIO10")
	(net 20 "GPIO20")
	(net 21 "GPIO21")

	;; ---------------------------------------------------------------------
	;; BOARD OUTLINE (70.0 mm x 46.0 mm, 3 mm corner radius)
	;; ---------------------------------------------------------------------
	(gr_line (start 3.0 0.0) (end 67.0 0.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 67.0 0.0) (mid 69.121 0.879) (end 70.0 3.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 70.0 3.0) (end 70.0 43.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 70.0 43.0) (mid 69.121 45.121) (end 67.0 46.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 67.0 46.0) (end 3.0 46.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 3.0 46.0) (mid 0.879 45.121) (end 0.0 43.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 0.0 43.0) (end 0.0 3.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 0.0 3.0) (mid 0.879 0.879) (end 3.0 0.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))

	;; ---------------------------------------------------------------------
	;; SILKSCREEN LABELS & HEADINGS
	;; ---------------------------------------------------------------------
	(gr_text "MultiBand 6 Atomic Beacon" (at 45.0 2.0) (layer "F.SilkS")
		(effects (font (size 1.0 1.0) (thickness 0.15) (bold yes))))
	(gr_text "Through-Hole Edition v1.0" (at 45.0 3.8) (layer "F.SilkS")
		(effects (font (size 0.65 0.65) (thickness 0.10))))
	(gr_text "USB-C" (at 16.12 3.0) (layer "F.SilkS")
		(effects (font (size 0.7 0.7) (thickness 0.12))))
	(gr_text "I2C / OLED" (at 12.81 43.8) (layer "F.SilkS")
		(effects (font (size 0.65 0.65) (thickness 0.10))))
	(gr_text "SYNC" (at 25.0 43.8) (layer "F.SilkS")
		(effects (font (size 0.65 0.65) (thickness 0.10) (bold yes))))
	(gr_text "EXT" (at 33.5 43.8) (layer "F.SilkS")
		(effects (font (size 0.6 0.6) (thickness 0.10))))

	;; ---------------------------------------------------------------------
	;; 4x M3 MOUNTING HOLES (3.2 mm drill, 6.0 mm copper pad, GND connected)
	;; ---------------------------------------------------------------------
	(footprint "MountingHole:MountingHole_3.2mm_M3" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 4.5 4.5)
		(pad "1" thru_hole circle (at 0 0) (size 6.0 6.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_3.2mm_M3" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 65.5 4.5)
		(pad "1" thru_hole circle (at 0 0) (size 6.0 6.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_3.2mm_M3" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 4.5 41.5)
		(pad "1" thru_hole circle (at 0 0) (size 6.0 6.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_3.2mm_M3" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 65.5 41.5)
		(pad "1" thru_hole circle (at 0 0) (size 6.0 6.0) (drill 3.2) (layers "*.Cu" "*.Mask") (net 1 "GND")))

	;; ---------------------------------------------------------------------
	;; U1: ESP32-C3 SuperMini Dual 1x8 Socket (2.54mm pitch, 600mil row spacing)
	;; ---------------------------------------------------------------------
	(footprint "Module:ESP32-C3-SuperMini_Socket" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 0 0)
		(property "Reference" "U1" (at 16.12 16.0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 1.0 1.0) (thickness 0.15) (bold yes))))
		(property "Value" "ESP32-C3-SuperMini" (at 16.12 18.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(fp_line (start 7.2 6.5) (end 25.0 6.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 25.0 6.5) (end 25.0 27.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 25.0 27.5) (end 7.2 27.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 7.2 27.5) (end 7.2 6.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 12.5 6.5) (end 12.5 2.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 12.5 2.5) (end 19.5 2.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 19.5 2.5) (end 19.5 6.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		;; Pin labels
		(fp_text user "5" (at 6.2 8.0) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "6" (at 6.2 10.54) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "7" (at 6.2 13.08) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "8" (at 6.2 15.62) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "9" (at 6.2 18.16) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "10" (at 6.2 20.70) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "20" (at 6.2 23.24) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "21" (at 6.2 25.78) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "5V" (at 21.8 8.0) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "G" (at 21.8 10.54) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "3V3" (at 21.8 13.08) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "4" (at 21.8 15.62) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "3" (at 21.8 18.16) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "2" (at 21.8 20.70) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "IO1" (at 21.8 23.24) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12) (bold yes))))
		(fp_text user "0" (at 21.8 25.78) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		;; Left Header Pads
		(pad "1" thru_hole rect (at 8.5 8.00) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 14 "GPIO5"))
		(pad "2" thru_hole oval (at 8.5 10.54) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 15 "GPIO6"))
		(pad "3" thru_hole oval (at 8.5 13.08) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 16 "GPIO7"))
		(pad "4" thru_hole oval (at 8.5 15.62) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 17 "GPIO8"))
		(pad "5" thru_hole oval (at 8.5 18.16) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 18 "GPIO9"))
		(pad "6" thru_hole oval (at 8.5 20.70) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 19 "GPIO10"))
		(pad "7" thru_hole oval (at 8.5 23.24) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 20 "GPIO20"))
		(pad "8" thru_hole oval (at 8.5 25.78) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 21 "GPIO21"))
		;; Right Header Pads
		(pad "9" thru_hole rect (at 23.74 8.00) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 2 "+5V"))
		(pad "10" thru_hole oval (at 23.74 10.54) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
		(pad "11" thru_hole oval (at 23.74 13.08) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
		(pad "12" thru_hole oval (at 23.74 15.62) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 13 "GPIO4"))
		(pad "13" thru_hole oval (at 23.74 18.16) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "14" thru_hole oval (at 23.74 20.70) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 11 "GPIO2"))
		(pad "15" thru_hole oval (at 23.74 23.24) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 3 "GPIO1_CARRIER"))
		(pad "16" thru_hole oval (at 23.74 25.78) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 10 "GPIO0"))
	)

	;; ---------------------------------------------------------------------
	;; C3: 10µF 25V Bulk Decoupling (Radial Electrolytic, D=5mm, P=2.54mm)
	;; ---------------------------------------------------------------------
	(footprint "Capacitor_THT:CP_Radial_D5.0mm_P2.50mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 29.5 5.5)
		(property "Reference" "C3" (at 0 -2.8) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "10uF" (at 0 2.8) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_circle (center 0 0) (end 2.5 0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_text user "+" (at -1.8 -1.27) (layer "F.SilkS") (effects (font (size 0.8 0.8) (thickness 0.12))))
		(pad "1" thru_hole rect (at 0 -1.27) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 2 "+5V"))
		(pad "2" thru_hole circle (at 0 1.27) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	;; ---------------------------------------------------------------------
	;; C4: 100nF Ceramic Bypass (Pitch 5.08mm)
	;; ---------------------------------------------------------------------
	(footprint "Capacitor_THT:C_Disc_D5.0mm_W2.5mm_P5.00mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 37.0 5.5)
		(property "Reference" "C4" (at 0 -2.0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "100nF" (at 0 2.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -2.2 -1.0) (end 2.2 -1.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 2.2 1.0) (end -2.2 1.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at -2.54 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 2 "+5V"))
		(pad "2" thru_hole circle (at 2.54 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	;; ---------------------------------------------------------------------
	;; R2: 220Ω 1/4W Damping Resistor (Axial DIN0207, P=10.16mm horizontal)
	;; ---------------------------------------------------------------------
	(footprint "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 10.0)
		(property "Reference" "R2" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "220R" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.0 -1.1) (end 3.0 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.0 -1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 -1.1) (end 3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 2 "+5V"))
		(pad "2" thru_hole circle (at 5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
	)

	;; ---------------------------------------------------------------------
	;; D1: 1N5819 Schottky Clamp Diode (DO-41 Axial, P=10.16mm horizontal)
	;; Cathode (Pin 1, rect) at right (+5V / Tank Hi side)
	;; Anode (Pin 2, circle) at left (Collector / Switched side)
	;; ---------------------------------------------------------------------
	(footprint "Diode_THT:D_DO-41_SOD81_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 13.5)
		(property "Reference" "D1" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "1N5819" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -2.4 -1.1) (end 2.4 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 2.4 1.1) (end -2.4 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -2.4 -1.1) (end -2.4 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 2.4 -1.1) (end 2.4 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.6 -1.1) (end 1.6 1.1) (stroke (width 0.25) (type solid)) (layer "F.SilkS"))
		(pad "2" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
		(pad "1" thru_hole rect (at 5.08 0) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
	)

	;; ---------------------------------------------------------------------
	;; R4: 4.7kΩ SDA Pull-Up Resistor (Axial DIN0207, P=10.16mm horizontal)
	;; Pad 1 (GPIO4) at left, Pad 2 (+3V3) at right
	;; ---------------------------------------------------------------------
	(footprint "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 17.0)
		(property "Reference" "R4" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "4.7k" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.0 -1.1) (end 3.0 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.0 -1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 -1.1) (end 3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 13 "GPIO4"))
		(pad "2" thru_hole circle (at 5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
	)

	;; ---------------------------------------------------------------------
	;; R6: 10kΩ SYNC Pull-Up Resistor (Axial DIN0207, P=10.16mm horizontal)
	;; Pad 1 (GPIO3) at left, Pad 2 (+3V3) at right
	;; ---------------------------------------------------------------------
	(footprint "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 20.5)
		(property "Reference" "R6" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "10k" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.0 -1.1) (end 3.0 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.0 -1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 -1.1) (end 3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "2" thru_hole circle (at 5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
	)

	;; ---------------------------------------------------------------------
	;; R1: 1kΩ Base Drive Resistor (Axial DIN0207, P=10.16mm horizontal)
	;; Pad 1 (GPIO2) at left, Pad 2 (NET_BASE) at right
	;; ---------------------------------------------------------------------
	(footprint "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 24.0)
		(property "Reference" "R1" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "1k" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.0 -1.1) (end 3.0 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.0 -1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 -1.1) (end 3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 3 "GPIO1_CARRIER"))
		(pad "2" thru_hole circle (at 5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 4 "NET_BASE"))
	)

	;; ---------------------------------------------------------------------
	;; R5: 4.7kΩ SCL Pull-Up Resistor (Axial DIN0207, P=10.16mm horizontal)
	;; Pad 1 (GPIO5) at left, Pad 2 (+3V3) at right
	;; ---------------------------------------------------------------------
	(footprint "Resistor_THT:R_Axial_DIN0207_L6.3mm_D2.5mm_P10.16mm_Horizontal" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 35.0 27.5)
		(property "Reference" "R5" (at 0 -1.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "4.7k" (at 0 1.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.0 -1.1) (end 3.0 -1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.0 -1.1) (end -3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.0 -1.1) (end 3.0 1.1) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 14 "GPIO5"))
		(pad "2" thru_hole circle (at 5.08 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
	)

	;; ---------------------------------------------------------------------
	;; Q1: C1815 / 2N2222 NPN Transistor (TO-92 Package, 2.54mm inline)
	;; Pin 2: Collector (NET_TANK_SW) at top (Y=21.46)
	;; Pin 3: Base (NET_BASE) at center (Y=24.0)
	;; Pin 1: Emitter (GND) at bottom (Y=26.54)
	;; ---------------------------------------------------------------------
	(footprint "Package_TO_SOT_THT:TO-92_Inline_Wide" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 46.0 24.0)
		(property "Reference" "Q1" (at 3.5 0 90) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10) (bold yes))))
		(property "Value" "C1815" (at 0 4.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -1.4 -3.0) (end -1.4 3.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_arc (start -1.4 3.0) (mid 1.8 0.0) (end -1.4 -3.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_text user "C" (at -2.5 -2.54) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12) (bold yes))))
		(fp_text user "B" (at -2.5 0.0) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12) (bold yes))))
		(fp_text user "E" (at -2.5 2.54) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12) (bold yes))))
		(pad "2" thru_hole circle (at 0 -2.54) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
		(pad "3" thru_hole circle (at 0 0) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 4 "NET_BASE"))
		(pad "1" thru_hole rect (at 0 2.54) (size 1.7 1.7) (drill 0.85) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	;; ---------------------------------------------------------------------
	;; C1: 1.5nF Base Tank Resonant Cap (THT Ceramic Disc / Film, P=5.08mm)
	;; ---------------------------------------------------------------------
	(footprint "Capacitor_THT:C_Rect_L7.2mm_W3.5mm_P5.08mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 51.5 14.5)
		(property "Reference" "C1" (at 0 -3.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "1.5nF" (at 0 3.4) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -1.4 -2.0) (end 1.4 -2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.4 -2.0) (end 1.4 2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.4 2.0) (end -1.4 2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -1.4 2.0) (end -1.4 -2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at 0 -2.54) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
		(pad "2" thru_hole circle (at 0 2.54) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
	)

	;; ---------------------------------------------------------------------
	;; C2: Optional THT Tuning Cap Socket (P=5.08mm, Box Film)
	;; ---------------------------------------------------------------------
	(footprint "Capacitor_THT:C_Rect_L7.2mm_W3.5mm_P5.08mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 56.5 14.5)
		(property "Reference" "C2" (at 0 -3.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "THT-CAP" (at 0 3.4) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -1.4 -2.0) (end 1.4 -2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.4 -2.0) (end 1.4 2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.4 2.0) (end -1.4 2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -1.4 2.0) (end -1.4 -2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at 0 -2.54) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
		(pad "2" thru_hole circle (at 0 2.54) (size 1.7 1.7) (drill 0.95) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
	)

	;; ---------------------------------------------------------------------
	;; J1: External Antenna Screw Terminal Block (KF301-2P, P=5.08mm)
	;; Placed safely at Y=14.5mm (Pad 1 at 11.96mm, well clear of M3 hole at Y=4.5)
	;; ---------------------------------------------------------------------
	(footprint "TerminalBlock:TerminalBlock_bornier-2_P5.08mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 63.5 14.5)
		(property "Reference" "J1" (at -4.0 0 90) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10) (bold yes))))
		(property "Value" "ANT_EXT" (at 0 4.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -3.4 -4.2) (end 3.8 -4.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.8 -4.2) (end 3.8 4.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.8 4.2) (end -3.4 4.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.4 4.2) (end -3.4 -4.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.4 -2.54) (end 2.0 -2.54) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.4 2.54) (end 2.0 2.54) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_text user "+" (at -1.8 -2.54) (layer "F.SilkS") (effects (font (size 0.75 0.75) (thickness 0.12))))
		(fp_text user "-" (at -1.8 2.54) (layer "F.SilkS") (effects (font (size 0.75 0.75) (thickness 0.12))))
		(pad "1" thru_hole rect (at 0 -2.54) (size 2.1 2.1) (drill 1.3) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
		(pad "2" thru_hole circle (at 0 2.54) (size 2.1 2.1) (drill 1.3) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
	)

	;; ---------------------------------------------------------------------
	;; L1: 16x18mm 3.5mH Drum Core Power Inductor (P=12.0mm, D=16mm)
	;; ---------------------------------------------------------------------
	(footprint "Inductor_THT:L_Radial_D16.0mm_P12.0mm_16x18_I-Type" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 57.0 31.0)
		(property "Reference" "L1" (at -3.6 -9.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.9 0.9) (thickness 0.14) (bold yes))))
		(fp_text user "EXT_ANT" (at 1.2 -9.4) (layer "F.SilkS") (effects (font (size 0.9 0.9) (thickness 0.14) (bold yes))))
		(property "Value" "16x18 3.5mH" (at 0 9.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_circle (center 0 0) (end 8.0 0) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_text user "I-CORE" (at 0 0) (layer "F.SilkS") (effects (font (size 0.9 0.9) (thickness 0.15))))
		(pad "1" thru_hole rect (at -6.0 0) (size 2.4 2.4) (drill 1.3) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
		(pad "2" thru_hole circle (at 6.0 0) (size 2.4 2.4) (drill 1.3) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
	)

	;; ---------------------------------------------------------------------
	;; J2: 1x4 Pin Header for I2C / OLED Display (P=2.54mm)
	;; ---------------------------------------------------------------------
	(footprint "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 12.81 40.0)
		(property "Reference" "J2" (at 0 -2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.65 0.65) (thickness 0.10) (bold yes))))
		(property "Value" "I2C_OLED" (at 0 2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -5.08 -1.27) (end 5.08 -1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 5.08 1.27) (end -5.08 1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_text user "G" (at -3.81 2.2) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12))))
		(fp_text user "3V" (at -1.27 2.2) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12))))
		(fp_text user "SCL" (at 1.27 2.2) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12))))
		(fp_text user "SDA" (at 3.81 2.2) (layer "F.SilkS") (effects (font (size 0.65 0.65) (thickness 0.12))))
		(pad "1" thru_hole rect (at -3.81 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
		(pad "2" thru_hole oval (at -1.27 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
		(pad "3" thru_hole oval (at 1.27 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 14 "GPIO5"))
		(pad "4" thru_hole oval (at 3.81 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 13 "GPIO4"))
	)

	;; ---------------------------------------------------------------------
	;; SW1: 6x6mm Tactile Pushbutton (Manual Sync Trigger)
	;; ---------------------------------------------------------------------
	(footprint "Button_Switch_THT:SW_PUSH_6mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 25.0 39.0)
		(property "Reference" "SW1" (at 0 -3.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.65 0.65) (thickness 0.10) (bold yes))))
		(property "Value" "SYNC_BTN" (at 0 3.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_circle (center 0 0) (end 1.5 0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -2.2 -2.0) (end 2.2 -2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 2.2 2.0) (end -2.2 2.0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole circle (at -3.25 -2.25) (size 1.6 1.6) (drill 1.1) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "2" thru_hole circle (at 3.25 -2.25) (size 1.6 1.6) (drill 1.1) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "3" thru_hole circle (at -3.25 2.25) (size 1.6 1.6) (drill 1.1) (layers "*.Cu" "*.Mask") (net 1 "GND"))
		(pad "4" thru_hole circle (at 3.25 2.25) (size 1.6 1.6) (drill 1.1) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	;; ---------------------------------------------------------------------
	;; J3: 1x2 Pin Header for External Sync Button (P=2.54mm)
	;; ---------------------------------------------------------------------
	(footprint "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 33.5 39.0)
		(property "Reference" "J3" (at 2.4 0 90) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.65 0.65) (thickness 0.10))))
		(property "Value" "EXT_SYNC" (at 0 2.4) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.5 0.5) (thickness 0.08))))
		(fp_line (start -1.27 -2.54) (end 1.27 -2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.27 2.54) (end -1.27 2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at 0 -1.27) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "2" thru_hole circle (at 0 1.27) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	;; ---------------------------------------------------------------------
	;; ROUTED COPPER TRACKS
	;; ---------------------------------------------------------------------

	;; Net 2 (+5V Power Rail - F.Cu 0.6mm)
	(segment (start 23.74 8.00) (end 26.5 8.00) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 8.00) (end 26.5 4.23) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 4.23) (end 29.5 4.23) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 29.5 4.23) (end 34.46 4.23) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 34.46 4.23) (end 34.46 5.5) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 8.00) (end 26.5 10.00) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 10.00) (end 29.92 10.00) (width 0.6) (layer "F.Cu") (net 2))

	;; Net 6 (NET_TANK_HI - F.Cu 0.6mm)
	(segment (start 40.08 10.0) (end 40.08 13.5) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 40.08 10.0) (end 51.5 10.0) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 51.5 10.0) (end 51.5 11.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 51.5 10.0) (end 56.5 10.0) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 56.5 10.0) (end 56.5 11.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 56.5 10.0) (end 63.5 10.0) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 63.5 10.0) (end 63.5 11.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 63.5 10.0) (end 66.5 10.0) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 66.5 10.0) (end 66.5 28.0) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 66.5 28.0) (end 63.0 31.0) (width 0.6) (layer "F.Cu") (net 6))

	;; Net 5 (NET_TANK_SW - 0.6mm)
	(segment (start 46.0 21.46) (end 46.0 17.04) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 46.0 17.04) (end 51.5 17.04) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 51.5 17.04) (end 56.5 17.04) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 56.5 17.04) (end 63.5 17.04) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 48.0 17.04) (end 48.0 29.0) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 48.0 29.0) (end 51.0 31.0) (width 0.6) (layer "F.Cu") (net 5))
	;; To D1 Anode at (29.92, 13.5) on B.Cu at Y=12.0mm (completely clear of GPIO4)
	(via (at 44.5 17.04) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 5))
	(segment (start 46.0 17.04) (end 44.5 17.04) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 44.5 17.04) (end 44.5 12.0) (width 0.6) (layer "B.Cu") (net 5))
	(segment (start 44.5 12.0) (end 29.92 12.0) (width 0.6) (layer "B.Cu") (net 5))
	(segment (start 29.92 12.0) (end 29.92 13.5) (width 0.6) (layer "B.Cu") (net 5))

	;; Net 4 (NET_BASE - F.Cu 0.4mm)
	(segment (start 40.08 24.0) (end 46.0 24.0) (width 0.4) (layer "F.Cu") (net 4))

	;; Net 3 (GPIO1_CARRIER - F.Cu 0.4mm from ESP32 Pin 15 at 23.74, 23.24)
	(segment (start 23.74 23.24) (end 25.5 23.24) (width 0.4) (layer "F.Cu") (net 3))
	(segment (start 25.5 23.24) (end 26.5 24.0) (width 0.4) (layer "F.Cu") (net 3))
	(segment (start 26.5 24.0) (end 29.92 24.0) (width 0.4) (layer "F.Cu") (net 3))

	;; Net 7 (+3V3 Rail: ESP32 Pin 3 -> J2 Pin 2 on F.Cu -> R5 Pad 2 on F.Cu -> R6 & R4 on B.Cu)
	;; To J2 Pin 2 via F.Cu inside ESP32 socket
	(segment (start 23.74 13.08) (end 11.54 13.08) (width 0.4) (layer "F.Cu") (net 7))
	(segment (start 11.54 13.08) (end 11.54 40.0) (width 0.4) (layer "F.Cu") (net 7))
	;; From J2 Pin 2 across bottom margin to R5 Pad 2 on F.Cu
	(segment (start 11.54 40.0) (end 11.54 43.5) (width 0.4) (layer "F.Cu") (net 7))
	(segment (start 11.54 43.5) (end 40.08 43.5) (width 0.4) (layer "F.Cu") (net 7))
	(segment (start 40.08 43.5) (end 40.08 27.5) (width 0.4) (layer "F.Cu") (net 7))
	;; From R5 Pad 2 (40.08, 27.5) to R6 Pad 2 (40.08, 20.5) and R4 Pad 2 (40.08, 17.0) on B.Cu
	(segment (start 40.08 27.5) (end 42.5 27.5) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 42.5 27.5) (end 42.5 20.5) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 42.5 20.5) (end 40.08 20.5) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 40.08 20.5) (end 40.08 17.0) (width 0.4) (layer "B.Cu") (net 7))

	;; Net 13 (GPIO4 / SDA)
	(segment (start 23.74 15.62) (end 29.92 15.62) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 29.92 15.62) (end 29.92 17.0) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 23.74 15.62) (end 16.62 15.62) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 16.62 15.62) (end 16.62 40.0) (width 0.35) (layer "F.Cu") (net 13))

	;; Net 12 (GPIO3 / SYNC)
	(segment (start 23.74 18.16) (end 28.0 18.16) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 28.0 18.16) (end 28.0 20.5) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 28.0 20.5) (end 29.92 20.5) (width 0.35) (layer "F.Cu") (net 12))
	;; To SW1 and J3 via F.Cu
	(segment (start 23.74 18.16) (end 21.75 18.16) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 21.75 18.16) (end 21.75 36.75) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 21.75 36.75) (end 28.25 36.75) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 28.25 36.75) (end 33.5 36.75) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 33.5 36.75) (end 33.5 37.73) (width 0.35) (layer "F.Cu") (net 12))

	;; Net 14 (GPIO5 / SCL)
	(segment (start 8.5 8.0) (end 5.5 8.0) (width 0.35) (layer "F.Cu") (net 14))
	(segment (start 5.5 8.0) (end 5.5 37.0) (width 0.35) (layer "F.Cu") (net 14))
	(via (at 5.5 37.0) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 14))
	(segment (start 5.5 37.0) (end 14.08 37.0) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 14.08 37.0) (end 14.08 40.0) (width 0.35) (layer "B.Cu") (net 14))
	;; To R5 Pad 1 via B.Cu (routed at Y=32.5mm well clear of SW1)
	(segment (start 14.08 37.0) (end 14.08 32.5) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 14.08 32.5) (end 28.5 32.5) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 28.5 32.5) (end 28.5 27.5) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 28.5 27.5) (end 29.92 27.5) (width 0.35) (layer "B.Cu") (net 14))

	;; ---------------------------------------------------------------------
	;; GROUND ZONE (B.Cu continuous ground pour covering entire board)
	;; ---------------------------------------------------------------------
	(zone (net 1) (net_name "GND") (layer "B.Cu") (uuid "{gen_uuid()}")
		(hatch edge 0.5)
		(priority 0)
		(connect_pads yes (clearance 0.25))
		(min_thickness 0.25)
		(fill yes (thermal_gap 0.3) (thermal_bridge_width 0.4))
		(polygon
			(pts
				(xy 0.5 0.5)
				(xy 69.5 0.5)
				(xy 69.5 45.5)
				(xy 0.5 45.5)
			)
		)
	)
)
"""
    clean_lines = [line for line in pcb_content.splitlines() if not line.strip().startswith(";;")]
    with open(pcb_path, "w", encoding="utf-8") as f:
        f.write("\n".join(clean_lines) + "\n")
    print(f"Created PCB layout: {pcb_path}")

# -------------------------------------------------------------------------
# 4. GENERATE BILL OF MATERIALS (bom.csv)
# -------------------------------------------------------------------------
def create_bom():
    bom_path = os.path.join(SCRIPT_DIR, "bom.csv")
    bom_content = """Designator,Value,Package / Footprint,Lead Spacing / Pitch,Description,Recommended Part / Type
U1,ESP32-C3 SuperMini,2x 1x8 Socket Header,P=2.54mm (600mil row spacing),RISC-V Wi-Fi/BLE MCU Module,Standard ESP32-C3 SuperMini
Q1,C1815 (or 2N2222),TO-92,P=2.54mm Inline,Small Signal NPN RF Switch Transistor,2SC1815 (E-C-B) or 2N2222 (E-B-C)
D1,1N5819,DO-41 Axial,P=10.16mm (400mil),1A 40V Schottky Inductive Clamp Diode,1N5819 (or 1N4148 / 1N4007)
R1,1k,Axial DIN0207,P=10.16mm (400mil),1/4W 5% Metal/Carbon Film Base Resistor,Standard 1k 1/4W Resistor
R2,220R,Axial DIN0207,P=10.16mm (400mil),1/4W 5% Metal/Carbon Film Damping Resistor,Standard 220R 1/4W Resistor
R4,4.7k,Axial DIN0207,P=10.16mm (400mil),1/4W 5% Metal Film I2C SDA Pull-Up,Standard 4.7k 1/4W Resistor
R5,4.7k,Axial DIN0207,P=10.16mm (400mil),1/4W 5% Metal Film I2C SCL Pull-Up,Standard 4.7k 1/4W Resistor
R6,10k,Axial DIN0207,P=10.16mm (400mil),1/4W 5% Metal Film Pushbutton Pull-Up,Standard 10k 1/4W Resistor
C1,1.5nF,Radial / Box Film,P=5.08mm (200mil),1.5nF (1500pF / code 152) 50V+ Tank Resonator,Metallized Film / C0G Ceramic
C2,Optional Socket,Box Film / Radial,P=5.08mm (200mil),Auxiliary Tank Tuning Capacitor Socket,Film / Ceramic Capacitor
C3,10uF,Radial Electrolytic (D=5mm),P=2.54mm (100mil),10uF 25V+ Bulk Decoupling Capacitor,Miniature Radial Electrolytic
C4,100nF,Radial Ceramic Disc / MLCC,P=5.08mm (200mil),100nF (0.1uF / code 104) 50V RF Bypass,Ceramic Disc / Multilayer Ceramic
L1,3.5mH 16x18,Radial Drum Inductor,P=12.0mm (D=16mm),16x18mm 3.5mH I-Type Open Core Radiator,Standard 1618 3.5mH Inductor
J1,KF301-2P,Screw Terminal Block,P=5.08mm,2-Pin Screw Terminal for External Antenna,Generic KF301-2P 5.08mm
SW1,6x6mm Switch,Tactile Pushbutton,6.5mm x 4.5mm pin spacing,Bedside Manual Sync Momentary Pushbutton,Standard 6x6x5mm Tactile Switch
J2,1x4 Header,Pin Header Male/Female,P=2.54mm,I2C Display Expansion Header (GND/3V3/SCL/SDA),Standard 2.54mm Header
J3,1x2 Header,Pin Header Male,P=2.54mm,External Pushbutton Header (Parallel with SW1),Standard 2.54mm Header
H1-H4,M3 Holes,Mounting Hole,D=3.2mm (6mm pad),Corner M3 Chassis / Standoff Mounting,Standard M3 Screws / Standoffs
"""
    with open(bom_path, "w", encoding="utf-8") as f:
        f.write(bom_content)
    print(f"Created BOM: {bom_path}")

# -------------------------------------------------------------------------
# 5. EXPORT GERBERS, DRILLS, DRC, AND 3D RENDERS
# -------------------------------------------------------------------------
def export_production():
    kicad_cli = shutil.which("kicad-cli")
    if not kicad_cli:
        for candidate in [
            "/Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli",
            "/Applications/KiCad 9.0/KiCad.app/Contents/MacOS/kicad-cli",
        ]:
            if os.path.exists(candidate):
                kicad_cli = candidate
                break

    if not kicad_cli:
        print("Note: kicad-cli not found. Skipping automated export.")
        return

    pcb_file = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}.kicad_pcb")
    gerber_dir = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}_Gerbers")
    if os.path.exists(gerber_dir):
        shutil.rmtree(gerber_dir)
    os.makedirs(gerber_dir, exist_ok=True)

    # 1. Refill zones and check DRC
    drc_report = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}-drc.rpt")
    subprocess.run([kicad_cli, "pcb", "drc", "--refill-zones", "--save-board", "-o", drc_report, pcb_file], check=False)

    # 2. Export Gerbers with Protel extensions and bottom-left auxiliary origin
    layers = "F.Cu,B.Cu,F.Mask,B.Mask,F.SilkS,B.SilkS,Edge.Cuts,F.Paste,B.Paste"
    cmd_gerber = [
        kicad_cli, "pcb", "export", "gerbers",
        "-o", gerber_dir,
        "--layers", layers,
        "--subtract-soldermask",
        "--check-zones",
        "--use-drill-file-origin",
        pcb_file
    ]
    subprocess.run(cmd_gerber, check=True)

    # 3. Export Excellon Drills (separate PTH & NPTH, metric decimal, plot origin)
    cmd_drill = [
        kicad_cli, "pcb", "export", "drill",
        "-o", gerber_dir,
        "--excellon-units", "mm",
        "--excellon-zeros-format", "decimal",
        "--excellon-oval-format", "alternate",
        "--excellon-separate-th",
        "--drill-origin", "plot",
        pcb_file
    ]
    subprocess.run(cmd_drill, check=True)

    # 4. Create ZIP package directly at root of archive
    zip_path = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}_Gerbers.zip")
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for root, _, files in os.walk(gerber_dir):
            for file in sorted(files):
                if file.startswith("."):
                    continue
                file_path = os.path.join(root, file)
                zf.write(file_path, arcname=file)

    print(f"Exported Gerbers & Drills: {gerber_dir}")
    print(f"Created Gerbers ZIP Package: {zip_path}")

    # 5. Render 3D preview images
    render_top = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}_Top.png")
    render_iso = os.path.join(SCRIPT_DIR, f"{PROJECT_NAME}_3D_Isometric.png")

    subprocess.run([
        kicad_cli, "pcb", "render",
        "--side", "top",
        "--width", "1600", "--height", "1050",
        "-o", render_top,
        pcb_file
    ], check=False)

    subprocess.run([
        kicad_cli, "pcb", "render",
        "--side", "top",
        "--rotate", "-40,0,35",
        "--perspective",
        "--width", "1600", "--height", "1050",
        "-o", render_iso,
        pcb_file
    ], check=False)

    print(f"Generated 3D Renders: {render_top}, {render_iso}")

if __name__ == "__main__":
    create_project()
    create_schematic()
    create_pcb()
    create_bom()
    export_production()
