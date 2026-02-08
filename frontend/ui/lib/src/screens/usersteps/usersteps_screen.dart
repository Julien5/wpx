import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/usersteps/usersteps_table.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class ButtonWidget extends StatelessWidget {
  const ButtonWidget({super.key});

  void gotoTable(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => UserStepsTableScreen(model: model),
      ),
    );
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

class _TextWidget extends StatelessWidget {
  const _TextWidget();

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

class UserStepsScaffold extends StatelessWidget {
  const UserStepsScaffold({super.key});

  @override
  Widget build(BuildContext ctx) {
    Set<InputType> usersteps = {InputType.userStep};

    List<Widget> midChilren = [
      _TextWidget(),
      UserStepsSliderProvider(),
      ButtonWidget(),
    ];
    return Scaffold(
      appBar: AppBar(title: const Text('Pacing Points')),
      body: AdaptiveLayout(
        topRow: TrackGraphicsRow(kinds: usersteps),
        midChildren: midChilren,
      ),
    );
  }
}

class UserStepsScreen extends StatelessWidget {
  const UserStepsScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return UserStepsScaffold();
  }
}
