import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/waypoints_table_widget.dart';

class UserStepsTableScaffold extends StatelessWidget {
  const UserStepsTableScaffold({super.key});

  /* void _setShortFormat(BuildContext ctx) {
    ParameterModel model = Provider.of<ParameterModel>(ctx, listen: false);
    model.setUserStepGpxNameFormat("TIME[%H:%M]");
  }

  void _setMediumFormat(BuildContext ctx) {
    ParameterModel model = Provider.of<ParameterModel>(ctx, listen: false);
    model.setUserStepGpxNameFormat("TIME[%H:%M]-SLOPE[4.1%]");
  }

  void _setLongFormat(BuildContext ctx) {
    ParameterModel model = Provider.of<ParameterModel>(ctx, listen: false);
    model.setUserStepGpxNameFormat("NAME[*]-TIME[%H:%M]-SLOPE[4.1%]");
  }*/

  @override
  Widget build(BuildContext ctx) {
    /*Widget shortButton = ElevatedButton(
      onPressed: () => _setShortFormat(ctx),
      child: const Text("short"),
    );

    Widget mediumButton = ElevatedButton(
      onPressed: () => _setMediumFormat(ctx),
      child: const Text("medium"),
    );

    Widget longButton = ElevatedButton(
      onPressed: () => _setLongFormat(ctx),
      child: const Text("long"),
    );

    Widget buttons = Card(
      elevation: 4, // Add shadow to the card
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(8), // Rounded corners
      ),
      child: Padding(
        padding: const EdgeInsets.all(50),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.spaceEvenly,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            shortButton,
            SizedBox(width: 10),
            mediumButton,
            SizedBox(width: 10),
            longButton,
          ],
        ),
      ),
    );
    */

    Widget column = Column(
      mainAxisAlignment: MainAxisAlignment.start,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        Divider(),
        /*buttons,
        SizedBox(height: 30),*/
        Expanded(
          child: Card(
            elevation: 4, // Add shadow to the card
            shape: RoundedRectangleBorder(
              borderRadius: BorderRadius.circular(8), // Rounded corners
            ),
            child: GPXTable(kinds: {Kind.cutOff}),
          ),
        ),
        Divider(),
        SizedBox(height: 30),
      ],
    );

    return Scaffold(
      appBar: AppBar(title: const Text('Cutoff Points Table')),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: 400,
          ), // Set max width to 400px
          child: column,
        ),
      ),
    );
  }
}

class UserStepsTableScreen extends StatelessWidget {
  final SegmentModel model;
  const UserStepsTableScreen({super.key, required this.model});

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider.value(
      value: model,
      builder: (innercontext, child) {
        return UserStepsTableScaffold();
      },
    );
  }
}
