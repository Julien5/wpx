import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_table.dart';
import 'package:ui/src/screens/wheel/wheel_screen.dart';

import '../../widgets/trackmultiview.dart';

class ButtonWidget extends StatelessWidget {
  const ButtonWidget({super.key});

  void gotoTable(BuildContext context) {
    SegmentModel model = Provider.of<SegmentModel>(context, listen: false);
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => ControlsTableScreen(model: model),
      ),
    );
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
            ScreenTrackView(kinds: control, height: 200),
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

class ControlsScreenProviders extends MultiProvider {
  ControlsScreenProviders({
    super.key,
    required SegmentModel segmentModel,
    required TrackMultiModel multiTrackModel,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider.value(value: segmentModel),
           ChangeNotifierProvider.value(value: multiTrackModel),
         ],
         child: child,
       );
}

class ControlsProvider extends StatelessWidget {
  final SegmentModel model;
  final TrackMultiModel multiTrackModel;
  const ControlsProvider({
    super.key,
    required this.model,
    required this.multiTrackModel,
  });

  @override
  Widget build(BuildContext context) {
    return ControlsScreenProviders(
      segmentModel: model,
      multiTrackModel: multiTrackModel,
      child: ControlsScreen(),
    );
  }
}
