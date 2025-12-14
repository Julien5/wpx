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
      child: const Text("GPX"),
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
    Set<InputType> control = {InputType.control};
    SegmentModel model = Provider.of<SegmentModel>(context);
    List<Waypoint> waypoints = model.someWaypoints(control);
    String text =
        waypoints.isEmpty ? "no controls" : "${waypoints.length} controls";
    return Center(child: Text(text));
  }
}

class ControlsScreen extends StatelessWidget {
  const ControlsScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> control = {InputType.control};

    return Scaffold(
      appBar: AppBar(title: const Text('Control Points')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Divider(),
            WheelWidget(kinds: control),
            Divider(),
            SizedBox(height: 10),
            TextWidget(),
            SizedBox(height: 10),
            Divider(),
            ButtonWidget(),
          ],
        ),
      ),
    );
  }
}

class ControlsProvider extends StatelessWidget {
  const ControlsProvider({super.key});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    Bridge bridge = root.getBridge();
    assert(bridge.isLoaded());
    return ChangeNotifierProvider(
      create: (context) => SegmentModel(bridge, root.trackSegment()),
      builder: (context, child) {
        return ControlsScreen();
      },
    );
  }
}
