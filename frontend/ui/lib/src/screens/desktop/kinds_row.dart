import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/kindsmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';

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

  void onUserStepsCheck(bool? checked) {
    KindsModel model = Provider.of(context, listen: false);
    if (checked == null || checked == false) {
      model.removeKind(Kind.userStep);
    } else {
      model.addKind(Kind.userStep);
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
    bool hasControls =
        model.kinds.contains(Kind.controls) && model.hasControls();
    bool hasGPXWaypoints =
        model.kinds.contains(Kind.gpxWaypoints) && model.hasGPXWaypoints();
    bool hasCities = model.kinds.contains(Kind.cities) && model.osmIsLoaded!;
    bool hasUserSteps = model.kinds.contains(Kind.userStep);
    SizedBox hdiv = SizedBox(width: 20);

    Function(bool?)? onControlCallback;
    if (model.hasControls()) {
      onControlCallback = onControlsCheck;
    }
    Function(bool?)? onWaypointsCallback;
    if (model.hasGPXWaypoints()) {
      onWaypointsCallback = onWaypointsCheck;
    }
    Function(bool?)? onOSMCallback;
    if (model.osmIsLoaded != null && model.osmIsLoaded!) {
      onOSMCallback = onOSMCheck;
    }
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        /*Checkbox(
          tristate: true,
          value: hasControls,
          onChanged: onControlCallback,
        ),
        Text("Controls"),
        hdiv,*/
        Checkbox(
          tristate: true,
          value: hasGPXWaypoints,
          onChanged: onWaypointsCallback,
        ),
        Text("Waypoints"),
        hdiv,
        Checkbox(tristate: true, value: hasCities, onChanged: onOSMCallback),
        Text("OSM"),
        hdiv,
        Checkbox(
          tristate: true,
          value: hasUserSteps,
          onChanged: onUserStepsCheck,
        ),
        Text("Pacing"),
      ],
    );
  }
}
