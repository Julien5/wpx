import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/segmentsgraphicsrow.dart';

class SettingsScreen extends StatelessWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext ctx) {
    Widget row = SegmentsGraphicsRow(kinds: {InputType.userStep}, height: 200);
    return Scaffold(
      appBar: AppBar(title: const Text('PDF')),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.start,
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [row, Divider(height: 5)],
        ),
      ),
    );
  }
}

class SettingsScreenProviders extends MultiProvider {
  SettingsScreenProviders({
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

class SettingsProvider extends StatelessWidget {
  final SegmentModel model;
  final TrackViewsSwitch multiTrackModel;
  const SettingsProvider({
    super.key,
    required this.model,
    required this.multiTrackModel,
  });

  @override
  Widget build(BuildContext context) {
    return SettingsScreenProviders(
      segmentModel: model,
      multiTrackModel: multiTrackModel,
      child: SettingsScreen(),
    );
  }
}
