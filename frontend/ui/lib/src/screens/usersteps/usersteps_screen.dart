import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/wheel/wheel_screen.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class UserStepsTable extends StatefulWidget {
  const UserStepsTable({super.key});

  @override
  State<UserStepsTable> createState() => _UserStepsTableState();
}

class _UserStepsTableState extends State<UserStepsTable> {
  late List<bridge.Waypoint> waypoints;

  @override
  Widget build(BuildContext context) {
    return Consumer<SegmentModel>(
      builder: (context, model, child) {
        Set<InputType> usersteps = {InputType.userStep};
        var waypoints = model.someWaypoints(usersteps);
        if (waypoints.isEmpty) {
          return const Text("No waypoints");
        }
        return Text("${waypoints.length} waypoints");
      },
    );
  }
}

class UserStepsScreen extends StatelessWidget {
  const UserStepsScreen({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  void goback(BuildContext ctx) {
    Navigator.of(ctx).pop();
  }

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> usersteps = {InputType.userStep};
    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points')),
      body: Center(
        child: Container(
          constraints: const BoxConstraints(maxWidth: 500),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              WheelWidget(kinds: usersteps),
              SizedBox(height: 50),
              UserStepsSliderProvider(),
              SizedBox(height: 50),
              UserStepsTable(),
            ],
          ),
        ),
      ),
    );
  }
}

class UserStepsProvider extends StatelessWidget {
  const UserStepsProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return ChangeNotifierProvider(
      create: (ctx) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return UserStepsScreen();
      },
    );
  }
}
