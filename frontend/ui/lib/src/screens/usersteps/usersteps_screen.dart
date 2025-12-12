import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class ButtonWidget extends StatelessWidget {
  const ButtonWidget({super.key});

  void gotoTable(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    Navigator.of(
      context,
    ).pushNamed(RouteManager.userStepsTable, arguments: model);
  }

  @override
  Widget build(BuildContext context) {
    Widget tableButton = ElevatedButton(
      onPressed: () => gotoTable(context),
      child: const Text("Table"),
    );
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [SizedBox(height: 10), tableButton, SizedBox(height: 10)],
    );
  }
}

class TextWidget extends StatelessWidget {
  const TextWidget({super.key});

  @override
  Widget build(BuildContext context) {
    Set<InputType> usersteps = {InputType.userStep};
    SegmentModel model = Provider.of<SegmentModel>(context);
    List<Waypoint> waypoints = model.someWaypoints(usersteps);
    String text =
        waypoints.isEmpty ? "no waypoints" : "${waypoints.length} waypoints";
    return Center(child: Text(text));
  }
}

class UserStepsScreen extends StatelessWidget {
  const UserStepsScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> usersteps = {InputType.userStep};

    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            WheelWidget(kinds: usersteps),
            Divider(),
            SizedBox(height: 10),
            TextWidget(),
            SizedBox(height: 10),
            UserStepsSliderProvider(),
            SizedBox(height: 10),
            Divider(),
            ButtonWidget(),
          ],
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
      create: (context) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return UserStepsScreen();
      },
    );
  }
}
