#!/usr/bin/env python3
"""
MultiBand6-ESP32-AtomicBeacon KiCad PCB & JLCPCB Manufacturing Files Generator
Creates:
  1. MultiBand6_Beacon.kicad_pro (KiCad Project File)
  2. MultiBand6_Beacon.kicad_sch (KiCad Schematic File)
  3. MultiBand6_Beacon.kicad_pcb (2-Layer PCB Layout: SMD + THT C2 + Screw Terminal J1)
  4. bom.csv                    (JLCPCB SMT Bill of Materials with Basic Part Numbers)
  5. positions.csv              (JLCPCB CPL Centroid Pick & Place File)
"""

import os
import uuid

PCB_DIR = os.path.dirname(os.path.abspath(__file__))

def gen_uuid():
    return str(uuid.uuid4())

# -------------------------------------------------------------------------
# 1. GENERATE KICAD PROJECT (.kicad_pro)
# -------------------------------------------------------------------------
def create_project():
    pro_path = os.path.join(PCB_DIR, "MultiBand6_Beacon.kicad_pro")
    if os.path.exists(pro_path) and os.path.getsize(pro_path) > 1000:
        print(f"Preserving existing KiCad 10 project settings: {pro_path}")
        return
    content = """{
  "board": {
    "design_settings": {
      "defaults": {
        "board_outline_line_width": 0.15,
        "copper_line_width": 0.25,
        "copper_text_size_h": 1.0,
        "copper_text_size_v": 1.0,
        "copper_text_thickness": 0.15,
        "silk_line_width": 0.15,
        "silk_text_size_h": 1.0,
        "silk_text_size_v": 1.0,
        "silk_text_thickness": 0.15
      },
      "rules": {
        "max_error": 0.005,
        "min_clearance": 0.2,
        "min_copper_edge_clearance": 0.3,
        "min_hole_clearance": 0.25,
        "min_hole_to_hole": 0.25,
        "min_through_hole_annular_ring": 0.15,
        "min_track_width": 0.2,
        "min_via_annular_ring": 0.15,
        "min_via_diameter": 0.6,
        "solder_mask_clearance": 0.05,
        "solder_mask_min_width": 0.1
      }
    }
  },
  "sheets": [
    [
      "MultiBand6_Beacon.kicad_sch"
    ]
  ]
}
"""
    with open(pro_path, "w", encoding="utf-8") as f:
        f.write(content)
    print(f"Created project: {pro_path}")

# -------------------------------------------------------------------------
# 2. GENERATE KICAD SCHEMATIC (.kicad_sch)
# -------------------------------------------------------------------------
def create_schematic():
    sch_path = os.path.join(PCB_DIR, "MultiBand6_Beacon.kicad_sch")
    sch_uuid = gen_uuid()
    
    content = f"""(kicad_sch
	(version 20260206)
	(generator "eeschema")
	(generator_version "10.0")
	(uuid "{sch_uuid}")
	(paper "A4")
	(title_block
		(title "MultiBand 6 Atomic Clock Beacon - ESP32-C3 Carrier")
		(date "2026-10-13")
		(rev "v1.1")
		(company "MultiBand6-ESP32-AtomicBeacon (@thomaslr)")
		(comment 1 "Optimized for JLCPCB SMT Basic Components ($0 setup fee)")
		(comment 2 "THT C2 tuning capacitor + KF301-2P screw terminal external antenna")
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
    pcb_path = os.path.join(PCB_DIR, "MultiBand6_Beacon.kicad_pcb")

    # Dimensions: 58.0 mm x 36.0 mm with 2.5 mm rounded corners
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
		(title "MultiBand 6 Atomic Clock Beacon - ESP32-C3 Carrier")
		(date "2026-10-13")
		(rev "v1.1")
		(company "MultiBand6-ESP32-AtomicBeacon (@thomaslr)")
		(comment 1 "Optimized for JLCPCB SMT Basic Components ($0 setup fee)")
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
		(aux_axis_origin 0 36)
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

	(gr_line (start 2.5 0.0) (end 55.5 0.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 55.5 0.0) (mid 57.268 0.732) (end 58.0 2.5) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 58.0 2.5) (end 58.0 33.5) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 58.0 33.5) (mid 57.268 35.268) (end 55.5 36.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 55.5 36.0) (end 2.5 36.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 2.5 36.0) (mid 0.732 35.268) (end 0.0 33.5) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_line (start 0.0 33.5) (end 0.0 2.5) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))
	(gr_arc (start 0.0 2.5) (mid 0.732 0.732) (end 2.5 0.0) (stroke (width 0.15) (type solid)) (layer "Edge.Cuts"))

	(gr_text "MultiBand 6 Atomic Beacon" (at 39.0 1.3) (layer "F.SilkS")
		(effects (font (size 1.0 1.0) (thickness 0.15) (bold yes))))
	(gr_text "ESP32-C3 Carrier v1.2" (at 35.0 34.5) (layer "F.SilkS")
		(effects (font (size 0.7 0.7) (thickness 0.12))))
	(gr_text "I2C / OLED" (at 12.81 33.5) (layer "F.SilkS")
		(effects (font (size 0.7 0.7) (thickness 0.12))))
	(gr_text "SYNC" (at 23.5 34.5) (layer "F.SilkS")
		(effects (font (size 0.7 0.7) (thickness 0.12) (bold yes))))
	(gr_text "EXT" (at 29.5 33.5) (layer "F.SilkS")
		(effects (font (size 0.6 0.6) (thickness 0.10))))
	(gr_text "USB-C" (at 16.12 3.0) (layer "F.SilkS")
		(effects (font (size 0.7 0.7) (thickness 0.12))))

	(footprint "MountingHole:MountingHole_2.7mm_M2.5" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 3.5 3.5)
		(pad "1" thru_hole circle (at 0 0) (size 4.0 4.0) (drill 2.7) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_2.7mm_M2.5" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 54.5 3.5)
		(pad "1" thru_hole circle (at 0 0) (size 4.0 4.0) (drill 2.7) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_2.7mm_M2.5" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 3.5 32.5)
		(pad "1" thru_hole circle (at 0 0) (size 4.0 4.0) (drill 2.7) (layers "*.Cu" "*.Mask") (net 1 "GND")))
	(footprint "MountingHole:MountingHole_2.7mm_M2.5" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 54.5 32.5)
		(pad "1" thru_hole circle (at 0 0) (size 4.0 4.0) (drill 2.7) (layers "*.Cu" "*.Mask") (net 1 "GND")))

	(footprint "Module:ESP32-C3-SuperMini_Socket" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 0 0)
		(property "Reference" "U1" (at 16.12 15.0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 1.0 1.0) (thickness 0.15) (bold yes))))
		(property "Value" "ESP32-C3-SuperMini" (at 16.12 17.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(fp_line (start 7.2 4.5) (end 25.0 4.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 25.0 4.5) (end 25.0 25.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 25.0 25.5) (end 7.2 25.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 7.2 25.5) (end 7.2 4.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 12.5 4.5) (end 12.5 1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 12.5 1.5) (end 19.5 1.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 19.5 1.5) (end 19.5 4.5) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_text user "5" (at 6.2 6.0) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "6" (at 6.2 8.54) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "7" (at 6.2 11.08) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "8" (at 6.2 13.62) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "9" (at 6.2 16.16) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "10" (at 6.2 18.70) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "20" (at 6.2 21.24) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "21" (at 6.2 23.78) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "5V" (at 21.8 6.0) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "G" (at 21.8 8.54) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "3V3" (at 21.8 11.08) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "4" (at 21.8 13.62) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "3" (at 21.8 16.16) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "2" (at 21.8 18.70) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(fp_text user "IO1" (at 21.8 21.24) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12) (bold yes))))
		(fp_text user "0" (at 21.8 23.78) (layer "F.SilkS") (effects (font (size 0.7 0.7) (thickness 0.12))))
		(pad "1" thru_hole rect (at 8.5 6.00) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 14 "GPIO5"))
		(pad "2" thru_hole oval (at 8.5 8.54) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 15 "GPIO6"))
		(pad "3" thru_hole oval (at 8.5 11.08) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 16 "GPIO7"))
		(pad "4" thru_hole oval (at 8.5 13.62) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 17 "GPIO8"))
		(pad "5" thru_hole oval (at 8.5 16.16) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 18 "GPIO9"))
		(pad "6" thru_hole oval (at 8.5 18.70) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 19 "GPIO10"))
		(pad "7" thru_hole oval (at 8.5 21.24) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 20 "GPIO20"))
		(pad "8" thru_hole oval (at 8.5 23.78) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 21 "GPIO21"))
		(pad "9" thru_hole rect (at 23.74 6.00) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 2 "+5V"))
		(pad "10" thru_hole oval (at 23.74 8.54) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
		(pad "11" thru_hole oval (at 23.74 11.08) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
		(pad "12" thru_hole oval (at 23.74 13.62) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 13 "GPIO4"))
		(pad "13" thru_hole oval (at 23.74 16.16) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "14" thru_hole oval (at 23.74 18.70) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 11 "GPIO2"))
		(pad "15" thru_hole oval (at 23.74 21.24) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 3 "GPIO1_CARRIER"))
		(pad "16" thru_hole oval (at 23.74 23.78) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 10 "GPIO0"))
	)

	(footprint "Capacitor_SMD:C_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 28.0 4.0)
		(property "Reference" "C3" (at 0 2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "Value" "10uF" (at 0 -2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C15850" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 2 "+5V"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 1 "GND"))
	)

	(footprint "Capacitor_SMD:C_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 31.5 4.0)
		(property "Reference" "C4" (at 0 2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "Value" "100nF" (at 0 -2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C49678" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 2 "+5V"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 1 "GND"))
	)

	(footprint "Resistor_SMD:R_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 28.0 10.0)
		(property "Reference" "R2" (at -1.6 0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(property "Value" "220R" (at 1.6 0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C17557" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 2 "+5V"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 6 "NET_TANK_HI"))
	)

	(footprint "Diode_SMD:D_SOD-123" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 31.5 12.0)
		(property "Reference" "D1" (at 1.6 0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(property "Value" "B5819W" (at -1.6 0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C8598" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(fp_line (start -0.6 -0.6) (end 0.6 -0.6) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(pad "1" smd roundrect (at 0 -1.6) (size 0.9 1.2) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 6 "NET_TANK_HI"))
		(pad "2" smd roundrect (at 0 1.6) (size 0.9 1.2) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 5 "NET_TANK_SW"))
	)

	(footprint "Capacitor_SMD:C_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 34.5 12.0)
		(property "Reference" "C1" (at 1.6 0) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(property "Value" "1.5nF" (at -1.6 0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C1717" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 6 "NET_TANK_HI"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 5 "NET_TANK_SW"))
	)

	(footprint "Capacitor_THT:C_Rect_L7.2mm_W3.5mm_P5.08mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 39.0 12.0)
		(property "Reference" "C2" (at 0 -4.6) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12) (bold yes))))
		(property "Value" "THT-CAP" (at 0 4.6) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(fp_line (start -1.8 -3.6) (end 1.8 -3.6) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.8 -3.6) (end 1.8 3.6) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.8 3.6) (end -1.8 3.6) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -1.8 3.6) (end -1.8 -3.6) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at 0 -2.54) (size 1.8 1.8) (drill 1.0) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
		(pad "2" thru_hole oval (at 0 2.54) (size 1.8 1.8) (drill 1.0) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
	)

	(footprint "TerminalBlock:TerminalBlock_bornier-2_P5.08mm" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 46.5 8.5)
		(property "Reference" "J1" (at -4.5 0 90) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12) (bold yes))))
		(property "Value" "ANT_EXT" (at 0 3.8) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(fp_line (start -3.8 -5.2) (end 4.2 -5.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 4.2 -5.2) (end 4.2 5.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 4.2 5.2) (end -3.8 5.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -3.8 5.2) (end -3.8 -5.2) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.8 -2.54) (end 2.2 -2.54) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_line (start 3.8 2.54) (end 2.2 2.54) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_text user "+" (at -2.0 -2.54) (layer "F.SilkS") (effects (font (size 0.8 0.8) (thickness 0.15))))
		(fp_text user "-" (at -2.0 2.54) (layer "F.SilkS") (effects (font (size 0.8 0.8) (thickness 0.15))))
		(pad "1" thru_hole rect (at 0 -2.54) (size 2.3 2.3) (drill 1.3) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
		(pad "2" thru_hole circle (at 0 2.54) (size 2.3 2.3) (drill 1.3) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
	)

	(footprint "Inductor_THT:L_Radial_D16.0mm_P12.0mm_16x18_I-Type" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 45.0 25.0)
		(property "Reference" "L1" (at -1.5 -9.8) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.85 0.85) (thickness 0.14) (bold yes))))
		(fp_text user "EXT_ANT" (at 2.2 -9.8) (layer "F.SilkS") (effects (font (size 0.85 0.85) (thickness 0.14) (bold yes))))
		(property "Value" "16x18 3.5mH" (at 0 9.8) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(fp_circle (center 0 0) (end 8.0 0) (stroke (width 0.2) (type solid)) (layer "F.SilkS"))
		(fp_text user "I-CORE" (at 0 0) (layer "F.SilkS") (effects (font (size 0.9 0.9) (thickness 0.15))))
		(pad "1" thru_hole rect (at -6.0 0) (size 2.5 2.5) (drill 1.3) (layers "*.Cu" "*.Mask") (net 5 "NET_TANK_SW"))
		(pad "2" thru_hole circle (at 6.0 0) (size 2.5 2.5) (drill 1.3) (layers "*.Cu" "*.Mask") (net 6 "NET_TANK_HI"))
	)

	(footprint "Resistor_SMD:R_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 27.5 18.70)
		(property "Reference" "R1" (at 0 -1.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(property "Value" "1k" (at 0 1.4) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C17513" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at -0.95 0) (size 1.0 1.3) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 3 "GPIO1_CARRIER"))
		(pad "2" smd roundrect (at 0.95 0) (size 1.0 1.3) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 4 "NET_BASE"))
	)

	(footprint "Package_TO_SOT_SMD:SOT-23" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 32.0 18.70)
		(property "Reference" "Q1" (at 0 -2.3) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.8 0.8) (thickness 0.12))))
		(property "Value" "MMBT2222A" (at 0 2.4) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "LCSC" "C8512" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(fp_line (start -1.5 -0.3) (end -1.5 0.3) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.5 -0.3) (end 1.5 0.3) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(pad "1" smd roundrect (at -0.95 1.0) (size 0.8 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 4 "NET_BASE"))
		(pad "2" smd roundrect (at 0.95 1.0) (size 0.8 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 1 "GND"))
		(pad "3" smd roundrect (at 0.0 -1.0) (size 0.8 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 5 "NET_TANK_SW"))
	)

	(footprint "Connector_PinHeader_2.54mm:PinHeader_1x04_P2.54mm_Vertical" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 12.81 30.5)
		(property "Reference" "J2" (at 0 -2.5) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "Value" "I2C_OLED" (at 0 2.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(fp_line (start -5.08 -1.27) (end 5.08 -1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 5.08 -1.27) (end 5.08 1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 5.08 1.27) (end -5.08 1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -5.08 1.27) (end -5.08 -1.27) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_text user "G" (at -3.81 2.2) (layer "F.SilkS") (effects (font (size 0.6 0.6) (thickness 0.10))))
		(fp_text user "3V" (at -1.27 2.2) (layer "F.SilkS") (effects (font (size 0.6 0.6) (thickness 0.10))))
		(fp_text user "SCL" (at 1.27 2.2) (layer "F.SilkS") (effects (font (size 0.6 0.6) (thickness 0.10))))
		(fp_text user "SDA" (at 3.81 2.2) (layer "F.SilkS") (effects (font (size 0.6 0.6) (thickness 0.10))))
		(pad "1" thru_hole rect (at -3.81 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
		(pad "2" thru_hole oval (at -1.27 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 7 "+3V3"))
		(pad "3" thru_hole oval (at 1.27 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 14 "GPIO5"))
		(pad "4" thru_hole oval (at 3.81 0) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 13 "GPIO4"))
	)

	(footprint "Button_Switch_SMD:SW_Push_1P1T_XKB_TS-1187A" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 23.5 30.5)
		(property "Reference" "SW1" (at 0 -3.4) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.11) (bold yes))))
		(property "Value" "TS-1187A" (at 0 0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "LCSC" "C318884" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(attr smd)
		(fp_line (start -1.8 -2.55) (end 1.8 -2.55) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(fp_line (start -1.8 2.55) (end 1.8 2.55) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(fp_line (start -2.55 -0.8) (end -2.55 0.8) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(fp_line (start 2.55 -0.8) (end 2.55 0.8) (stroke (width 0.12) (type solid)) (layer "F.SilkS"))
		(fp_circle (center 0 0) (end 1.5 0) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" smd rect (at -3 -1.875) (size 1.1 0.8) (layers "F.Cu" "F.Paste" "F.Mask") (net 12 "GPIO3"))
		(pad "1" smd rect (at 3 -1.875) (size 1.1 0.8) (layers "F.Cu" "F.Paste" "F.Mask") (net 12 "GPIO3"))
		(pad "2" smd rect (at -3 1.875) (size 1.1 0.8) (layers "F.Cu" "F.Paste" "F.Mask") (net 1 "GND"))
		(pad "2" smd rect (at 3 1.875) (size 1.1 0.8) (layers "F.Cu" "F.Paste" "F.Mask") (net 1 "GND"))
	)

	(footprint "Connector_PinHeader_2.54mm:PinHeader_1x02_P2.54mm_Vertical" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 29.5 30.5)
		(property "Reference" "J3" (at -2.2 0 90) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.7 0.7) (thickness 0.10))))
		(property "Value" "EXT_SYNC" (at 0 2.0) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(fp_line (start -1.27 -2.54) (end 1.27 -2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.27 -2.54) (end 1.27 2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start 1.27 2.54) (end -1.27 2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(fp_line (start -1.27 2.54) (end -1.27 -2.54) (stroke (width 0.15) (type solid)) (layer "F.SilkS"))
		(pad "1" thru_hole rect (at 0 -1.27) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 12 "GPIO3"))
		(pad "2" thru_hole circle (at 0 1.27) (size 1.7 1.7) (drill 1.0) (layers "*.Cu" "*.Mask") (net 1 "GND"))
	)

	(footprint "Resistor_SMD:R_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 31.5 23.0)
		(property "Reference" "R4" (at 0 -2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "4.7k" (at 0 2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "LCSC" "C17673" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 7 "+3V3"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 13 "GPIO4"))
	)

	(footprint "Resistor_SMD:R_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 28.5 23.0)
		(property "Reference" "R5" (at 0 -2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "4.7k" (at 0 2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "LCSC" "C17673" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 7 "+3V3"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 14 "GPIO5"))
	)

	(footprint "Resistor_SMD:R_0805_2012Metric" (layer "F.Cu") (uuid "{gen_uuid()}")
		(at 34.5 23.0)
		(property "Reference" "R6" (at 0 -2.2) (layer "F.SilkS") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "Value" "10k" (at 0 2.2) (layer "F.Fab") (uuid "{gen_uuid()}")
			(effects (font (size 0.6 0.6) (thickness 0.10))))
		(property "LCSC" "C17414" (at 0 0 0) (layer "F.Fab") (hide yes) (uuid "{gen_uuid()}"))
		(pad "1" smd roundrect (at 0 -0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 7 "+3V3"))
		(pad "2" smd roundrect (at 0 0.95) (size 1.3 1.0) (layers "F.Cu" "F.Paste" "F.Mask") (roundrect_rratio 0.25) (net 12 "GPIO3"))
	)

	(via (at 29.75 5.0) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 34.5 19.70) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 36.5 5.0) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 36.5 19.70) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 36.5 32.375) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 23.5 33.50) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 9.0 33.50) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 3.5 18.0) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 1))
	(via (at 26.8 22.05) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 7))
	(via (at 26.8 23.95) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 14))
	(via (at 33.0 23.95) (size 0.8) (drill 0.4) (layers "F.Cu" "B.Cu") (net 13))

	(segment (start 23.74 6.00) (end 26.5 6.00) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 6.00) (end 26.5 3.05) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 3.05) (end 32.0 3.05) (width 0.6) (layer "F.Cu") (net 2))
	(segment (start 26.5 6.00) (end 26.5 9.05) (width 0.5) (layer "F.Cu") (net 2))
	(segment (start 26.5 9.05) (end 28.0 9.05) (width 0.6) (layer "F.Cu") (net 2))

	(segment (start 28.0 10.95) (end 31.5 10.4) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 31.5 10.4) (end 34.5 11.05) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 34.5 11.05) (end 39.0 9.46) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 39.0 9.46) (end 39.0 5.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 39.0 5.96) (end 46.5 5.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 46.5 5.96) (end 51.5 5.96) (width 0.6) (layer "F.Cu") (net 6))
	(segment (start 51.5 5.96) (end 51.5 25.0) (width 0.5) (layer "F.Cu") (net 6))
	(segment (start 51.5 25.0) (end 51.0 25.0) (width 0.6) (layer "F.Cu") (net 6))

	(segment (start 31.5 13.6) (end 31.5 14.54) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 34.5 12.95) (end 34.5 14.54) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 32.0 17.70) (end 32.0 14.54) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 31.5 14.54) (end 46.5 14.54) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 39.0 14.54) (end 39.0 25.0) (width 0.6) (layer "F.Cu") (net 5))
	(segment (start 46.5 14.54) (end 46.5 11.04) (width 0.6) (layer "F.Cu") (net 5))

	;; Net 3 (GPIO1_CARRIER - F.Cu 0.4mm from ESP32 Pin 15 at 23.74, 21.24)
	(segment (start 23.74 21.24) (end 25.2 21.24) (width 0.4) (layer "F.Cu") (net 3))
	(segment (start 25.2 21.24) (end 25.2 18.70) (width 0.4) (layer "F.Cu") (net 3))
	(segment (start 25.2 18.70) (end 26.55 18.70) (width 0.4) (layer "F.Cu") (net 3))

	(segment (start 28.45 18.70) (end 30.0 18.70) (width 0.4) (layer "F.Cu") (net 4))
	(segment (start 30.0 18.70) (end 31.0 19.70) (width 0.4) (layer "F.Cu") (net 4))
	(segment (start 31.0 19.70) (end 31.05 19.70) (width 0.4) (layer "F.Cu") (net 4))

	(segment (start 28.0 4.95) (end 29.75 4.95) (width 0.4) (layer "F.Cu") (net 1))
	(segment (start 29.75 4.95) (end 29.75 5.0) (width 0.4) (layer "F.Cu") (net 1))
	(segment (start 31.5 4.95) (end 29.75 4.95) (width 0.4) (layer "F.Cu") (net 1))
	(segment (start 32.95 19.70) (end 34.5 19.70) (width 0.4) (layer "F.Cu") (net 1))

	(segment (start 26.8 22.05) (end 28.5 22.05) (width 0.35) (layer "F.Cu") (net 7))
	(segment (start 28.5 22.05) (end 31.5 22.05) (width 0.35) (layer "F.Cu") (net 7))
	(segment (start 31.5 22.05) (end 34.5 22.05) (width 0.35) (layer "F.Cu") (net 7))
	(segment (start 23.74 11.08) (end 11.54 11.08) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 11.54 11.08) (end 11.54 30.5) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 23.74 11.08) (end 25.5 11.08) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 25.5 11.08) (end 25.5 22.05) (width 0.4) (layer "B.Cu") (net 7))
	(segment (start 25.5 22.05) (end 26.8 22.05) (width 0.4) (layer "B.Cu") (net 7))

	(segment (start 8.5 6.00) (end 14.08 6.00) (width 0.35) (layer "F.Cu") (net 14))
	(segment (start 14.08 6.00) (end 14.08 30.5) (width 0.35) (layer "F.Cu") (net 14))
	(segment (start 28.5 23.95) (end 26.8 23.95) (width 0.35) (layer "F.Cu") (net 14))
	(segment (start 26.8 23.95) (end 26.8 25.3) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 26.8 25.3) (end 14.08 25.3) (width 0.35) (layer "B.Cu") (net 14))
	(segment (start 14.08 25.3) (end 14.08 30.5) (width 0.35) (layer "B.Cu") (net 14))

	(segment (start 23.74 13.62) (end 16.62 13.62) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 16.62 13.62) (end 16.62 30.5) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 31.5 23.95) (end 33.0 23.95) (width 0.35) (layer "F.Cu") (net 13))
	(segment (start 33.0 23.95) (end 33.0 26.3) (width 0.35) (layer "B.Cu") (net 13))
	(segment (start 33.0 26.3) (end 16.62 26.3) (width 0.35) (layer "B.Cu") (net 13))
	(segment (start 16.62 26.3) (end 16.62 30.5) (width 0.35) (layer "B.Cu") (net 13))

	(segment (start 23.74 16.16) (end 21.25 16.16) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 21.25 16.16) (end 21.25 26.0) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 21.25 26.0) (end 20.5 26.75) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 20.5 26.75) (end 20.5 28.625) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 20.5 28.625) (end 26.5 28.625) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 26.5 28.625) (end 29.5 28.625) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 29.5 28.625) (end 29.5 29.23) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 21.25 26.0) (end 34.5 26.0) (width 0.35) (layer "F.Cu") (net 12))
	(segment (start 34.5 26.0) (end 34.5 23.95) (width 0.35) (layer "F.Cu") (net 12))

	;; ---------------------------------------------------------------------
	;; Net 1 (GND) - Master Continuous Ground Trunks & Stitching Network
	;; ---------------------------------------------------------------------
	;; 1. Bottom Ground Bus (joining J2 OLED GND, SW1 GND, J3 EXT GND)
	(segment (start 9.00 30.50) (end 9.00 32.375) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 9.00 32.375) (end 20.5 32.375) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 20.5 32.375) (end 26.5 32.375) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 26.5 32.375) (end 29.5 32.375) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 29.5 32.375) (end 29.5 31.77) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 9.00 32.375) (end 9.00 33.50) (width 0.4) (layer "F.Cu") (net 1))
	(segment (start 23.5 32.375) (end 23.5 33.50) (width 0.4) (layer "F.Cu") (net 1))

	;; 2. Right Ground Trunk (connecting bottom bus directly to Q1 Emitter & ESP32 GND)
	(segment (start 29.5 32.375) (end 36.5 32.375) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 36.5 32.375) (end 36.5 19.70) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 36.5 19.70) (end 34.5 19.70) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 29.5 32.375) (end 36.5 32.375) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 36.5 32.375) (end 36.5 19.70) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 36.5 19.70) (end 36.5 5.0) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 36.5 5.0) (end 29.75 5.0) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 29.75 5.0) (end 23.74 8.54) (width 0.6) (layer "B.Cu") (net 1))

	;; 3. Left Ground Corridor (connecting bottom bus to mounting holes)
	(segment (start 9.00 32.375) (end 3.5 32.5) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 3.5 32.5) (end 3.5 18.0) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 3.5 18.0) (end 3.5 3.5) (width 0.6) (layer "F.Cu") (net 1))
	(segment (start 3.5 32.5) (end 3.5 3.5) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 29.75 5.0) (end 29.75 8.54) (width 0.6) (layer "B.Cu") (net 1))
	(segment (start 29.75 8.54) (end 23.74 8.54) (width 0.6) (layer "B.Cu") (net 1))

	;; Ground Copper Pour (F.Cu - Top Layer)
	(zone (net 1) (net_name "GND") (layer "F.Cu") (uuid "{gen_uuid()}")
		(hatch edge 0.5)
		(priority 0)
		(connect_pads yes (clearance 0.3))
		(min_thickness 0.25)
		(fill yes (thermal_gap 0.3) (thermal_bridge_width 0.4))
		(polygon
			(pts
				(xy 0.5 0.5)
				(xy 57.5 0.5)
				(xy 57.5 35.5)
				(xy 0.5 35.5)
			)
		)
	)

	;; Ground Copper Pour (B.Cu - Bottom Layer)
	(zone (net 1) (net_name "GND") (layer "B.Cu") (uuid "{gen_uuid()}")
		(hatch edge 0.5)
		(priority 0)
		(connect_pads yes (clearance 0.3))
		(min_thickness 0.25)
		(fill yes (thermal_gap 0.3) (thermal_bridge_width 0.4))
		(polygon
			(pts
				(xy 0.5 0.5)
				(xy 57.5 0.5)
				(xy 57.5 35.5)
				(xy 0.5 35.5)
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
# 4. GENERATE JLCPCB SMT FILES (BOM & CPL)
# -------------------------------------------------------------------------
def create_jlcpcb_files():
    # 1. BOM file (JLCPCB SMT Format) - Written directly to PCB_DIR
    # Only verified JLCPCB BASIC components ($0.00 setup fee)
    bom_path = os.path.join(PCB_DIR, "bom.csv")
    bom_content = """Comment,Designator,Footprint,LCSC Part #,Type
1k,R1,R_0805_2012Metric,C17513,Basic
220R,R2,R_0805_2012Metric,C17557,Basic
1.5nF 50V,C1,C_0805_2012Metric,C1717,Basic
10uF 25V,C3,C_0805_2012Metric,C15850,Basic
100nF 50V,C4,C_0805_2012Metric,C49678,Basic
MMBT2222A,Q1,SOT-23,C8512,Basic
B5819W,D1,D_SOD-123,C8598,Basic
4.7k,R4,R_0805_2012Metric,C17673,Basic
4.7k,R5,R_0805_2012Metric,C17673,Basic
10k,R6,R_0805_2012Metric,C17414,Basic
TS-1187A,SW1,SW_Push_1P1T_XKB_TS-1187A,C318884,Basic
"""
    with open(bom_path, "w", encoding="utf-8") as f:
        f.write(bom_content)
    print(f"Created JLCPCB BOM: {bom_path}")

    # 2. CPL Centroid Position file (JLCPCB SMT Format) - Written directly to PCB_DIR
    # Coordinates use bottom-left board origin (0, 36.0 in KiCad canvas -> Y_jlc = 36 - Y_canvas)
    # Rotation angles calibrated for JLCPCB tape-and-reel feeding:
    # - Resistors/Caps (0805): 0.0 for horizontal (R1), 90.0 for vertical (C1, C3, C4, R2, R4, R5, R6)
    # - D1 (SOD-123): 90.0 (Cathode pointing up)
    # - Q1 (SOT-23): 90.0 (Collector Pin 3 pointing up, Base Pin 1 bottom-left, Emitter Pin 2 bottom-right)
    # - SW1 (TS-1187A): 0.0 (Standard tape-and-reel orientation)
    pos_path = os.path.join(PCB_DIR, "positions.csv")
    pos_content = """Designator,Val,Package,Mid X,Mid Y,Rotation,Layer
C3,10uF,0805,28.0,32.0,90.0,top
C4,100nF,0805,31.5,32.0,90.0,top
R2,220R,0805,28.0,26.0,90.0,top
D1,B5819W,SOD-123,31.5,24.0,90.0,top
C1,1.5nF,0805,34.5,24.0,90.0,top
R1,1k,0805,27.5,17.3,0.0,top
Q1,MMBT2222A,SOT-23,32.0,17.3,90.0,top
R4,4.7k,0805,31.5,13.0,90.0,top
R5,4.7k,0805,28.5,13.0,90.0,top
R6,10k,0805,34.5,13.0,90.0,top
SW1,TS-1187A,TS-1187A,23.5,5.5,0.0,top
"""
    with open(pos_path, "w", encoding="utf-8") as f:
        f.write(pos_content)
    print(f"Created JLCPCB CPL: {pos_path}")

def export_jlcpcb_production():
    """
    Exports Gerbers, Drills, and ZIP following official JLCPCB KiCad 9/10 instructions:
    https://jlcpcb.com/help/article/how-to-generate-gerber-and-drill-files-in-kicad-9
    """
    import shutil
    import subprocess
    import zipfile

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
        print("Note: kicad-cli not found in standard paths. Skipping automated export.")
        return

    pcb_file = os.path.join(PCB_DIR, "MultiBand6_Beacon.kicad_pcb")
    gerber_dir = os.path.join(PCB_DIR, "MultiBand6_Beacon_Gerbers")
    if os.path.exists(gerber_dir):
        shutil.rmtree(gerber_dir)
    os.makedirs(gerber_dir, exist_ok=True)

    # 1. Refill zones, save board, and save DRC report
    drc_report = os.path.join(PCB_DIR, "MultiBand6_Beacon-drc.rpt")
    subprocess.run([kicad_cli, "pcb", "drc", "--refill-zones", "--save-board", "-o", drc_report, pcb_file], check=False)

    # 2. Export Gerbers with Protel extensions and drill/aux origin
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

    # 3. Export Excellon Drills (separate PTH and NPTH, metric decimal, matching drill origin)
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

    # 4. Create ZIP package (all Gerber & Drill files placed directly at the root of the archive)
    zip_path = os.path.join(PCB_DIR, "MultiBand6_Beacon_Gerbers.zip")
    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        for root, _, files in os.walk(gerber_dir):
            for file in sorted(files):
                if file.startswith("."):
                    continue
                file_path = os.path.join(root, file)
                zf.write(file_path, arcname=file)

    print(f"Exported JLCPCB Gerbers & Drills: {gerber_dir}")
    print(f"Created JLCPCB Zip Package: {zip_path}")

    # 5. Render 3D preview images
    render_top = os.path.join(PCB_DIR, "MultiBand6_Beacon_Top.png")
    subprocess.run([
        kicad_cli, "pcb", "render",
        "-o", render_top,
        "--side", "top",
        "--width", "1200", "--height", "800",
        pcb_file
    ], check=False)

    render_iso = os.path.join(PCB_DIR, "MultiBand6_Beacon_3D_Isometric.png")
    subprocess.run([
        kicad_cli, "pcb", "render",
        "-o", render_iso,
        "--side", "top",
        "--rotate", "-45,0,45",
        "--floor", "--perspective",
        "--width", "1200", "--height", "800",
        pcb_file
    ], check=False)
    print(f"Generated 3D Renders: {render_top}, {render_iso}")

if __name__ == "__main__":
    create_project()
    create_schematic()
    create_pcb()
    create_jlcpcb_files()
    export_jlcpcb_production()
