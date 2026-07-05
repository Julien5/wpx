import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/routes.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/usersteps/usersteps_table.dart';
import 'package:wpx/src/widgets/adaptive_layout.dart';
import 'package:wpx/src/widgets/segmentgraphics.dart';
import 'package:wpx/src/widgets/userstepsslider.dart';

class ButtonWidget extends StatelessWidget {
  const ButtonWidget({super.key});

  void gotoTable(BuildContext context) {
    SegmentModel model = context.read<SegmentModel>();
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
    Kinds usersteps = [Kind.cutOff];
    SegmentModel model = context.watch<SegmentModel>();
    List<Waypoint> waypoints = model.someWaypoints(usersteps);
    String text =
        waypoints.isEmpty
            ? "no cutoff points"
            : "${waypoints.length} cutoff points";
    return Center(child: Text(text));
  }
}

class UserStepsScaffold extends StatelessWidget {
  const UserStepsScaffold({super.key});

  @override
  Widget build(BuildContext ctx) {
    Kinds usersteps = [Kind.cutOff];

    List<Widget> midChilren = [
      _TextWidget(),
      UserStepsSliderProvider(),
      ButtonWidget(),
    ];
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: Icon(Icons.arrow_back),
          onPressed: () => gotoOverview(ctx),
        ),
        title: const Text('Cutoff Points'),
      ),

      body: MobileScaffoldBody(
        topRow: TrackGraphicsRow(kinds: usersteps),
        midColumn: MidColumn(children: midChilren),
        label: 'usersteps',
        clients: [
          RenderFunction.profile,
          RenderFunction.map,
          RenderFunction.wheel,
        ],
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
