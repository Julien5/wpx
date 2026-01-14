import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/usersteps/usersteps_table.dart';
import 'package:ui/src/widgets/trackmultiview.dart';
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
            ScreenTrackView(kinds: usersteps, height: 200),
            SizedBox(height: 10),
            TextWidget(),
            SizedBox(height: 10),
            UserStepsSliderProvider(),
            SizedBox(height: 10),
            Divider(height: 5),
            ButtonWidget(),
          ],
        ),
      ),
    );
  }
}

class UserStepsScreenProviders extends MultiProvider {
  UserStepsScreenProviders({
    super.key,
    required SegmentModel segmentModel,
    required TrackViewsSwitch multiTrackModel,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider.value(value: segmentModel),
           ChangeNotifierProvider.value(value: multiTrackModel),
         ],
         child: child,
       );
}

class UserStepsProvider extends StatelessWidget {
  final SegmentModel model;
  final TrackViewsSwitch multiTrackModel;
  const UserStepsProvider({
    super.key,
    required this.model,
    required this.multiTrackModel,
  });

  @override
  Widget build(BuildContext context) {
    return UserStepsScreenProviders(
      segmentModel: model,
      multiTrackModel: multiTrackModel,
      child: UserStepsScreen(),
    );
  }
}
