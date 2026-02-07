import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/screens/controls/controls_screen.dart';
import 'package:ui/src/screens/settings/settings_screen.dart';
import 'package:ui/src/screens/wheel/statistics_widget.dart';
import 'package:ui/src/widgets/adaptive_layout.dart';
import 'package:ui/src/widgets/export.dart';
import 'package:ui/src/widgets/segmentgraphics.dart';

class _WheelScaffold extends StatelessWidget {
  void gotoSettings(BuildContext ctx) {
    SegmentModel model = Provider.of<SegmentModel>(ctx, listen: false);
    Navigator.push(
      ctx,
      MaterialPageRoute(builder: (context) => SettingsScreen(model: model)),
    );
  }

  void gotoUserSteps(BuildContext ctx) {
    // TODO: context.go to pacing
  }

  void gotoControls(BuildContext ctx) {
    TrackViewsSwitch viewsSwitch = Provider.of<TrackViewsSwitch>(
      ctx,
      listen: false,
    );
    Navigator.push(
      ctx,
      MaterialPageRoute(
        builder: (context) => ControlsScreen(switchModel: viewsSwitch),
      ),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    Widget statisticsCard = StatisticsWidget(
      onPacingPointPressed: () => gotoUserSteps(ctx),
      onControlsPointPressed: () => gotoControls(ctx),
      onPagesPressed: () => gotoSettings(ctx),
    );
    List<Widget> children = [
      statisticsCard,
      Center(child: ExportButton(text: "export zip", type: Type.zip)),
    ];

    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: AdaptiveLayout(
        topRow: TrackGraphicsRow(kinds: allkinds()),
        midChildren: children,
      ),
    );
  }
}

class _WheelScreenProviders extends MultiProvider {
  _WheelScreenProviders({required Widget child})
    : super(
        providers: [
          ChangeNotifierProvider(
            create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
          ),
        ],
        child: child,
      );
}

class WheelScreen extends StatelessWidget {
  const WheelScreen({super.key});

  @override
  Widget build(BuildContext context) {
    context.watch<SegmentModel>();
    return _WheelScreenProviders(child: _WheelScaffold());
  }
}
