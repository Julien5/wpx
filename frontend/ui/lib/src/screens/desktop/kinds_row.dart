import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/kindsmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';

class KindsRow extends StatefulWidget {
  const KindsRow({super.key});

  @override
  State<KindsRow> createState() => _KindsRowState();
}

class _KindsRowState extends State<KindsRow> {
  void onControlsCheck(bool? checked) {
    KindsModel model = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      model.removeKind(Kind.controls);
    } else {
      model.addKind(Kind.controls);
    }
  }

  void onWaypointsCheck(bool? checked) {
    KindsModel model = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      model.removeKind(Kind.gpxWaypoints);
    } else {
      model.addKind(Kind.gpxWaypoints);
    }
  }

  void onOSMCheck(bool? checked) {
    KindsModel model = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      model.removeOSM();
    } else {
      model.addOSM();
    }
  }

  @override
  Widget build(BuildContext context) {
    KindsModel model = Provider.of(context);
    bool hasControls = model.kinds.contains(Kind.controls);
    bool hasGPXWaypoints = model.kinds.contains(Kind.gpxWaypoints);
    bool hasCities = model.kinds.contains(Kind.cities);
    SizedBox hdiv = SizedBox(width: 10);
    return Row(
      children: [
        Checkbox(
          tristate: true,
          value: hasControls,
          onChanged: onControlsCheck,
        ),
        Text("Controls"),
        hdiv,
        Checkbox(
          tristate: true,
          value: hasGPXWaypoints,
          onChanged: onWaypointsCheck,
        ),
        Text("Waypoints"),
        hdiv,
        Checkbox(tristate: true, value: hasCities, onChanged: onOSMCheck),
        Text("OSM"),
      ],
    );
  }
}