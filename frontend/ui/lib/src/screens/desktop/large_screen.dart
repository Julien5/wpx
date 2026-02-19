import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/models/screen_configuration.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/models/trackviewswitch.dart';
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/trackview.dart';
import 'package:ui/src/widgets/userstepsslider.dart';

class GraphicsPadding extends StatelessWidget {
  final Widget child;

  const GraphicsPadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(padding: EdgeInsetsGeometry.all(20), child: child);
  }
}

class ProfilePadding extends StatelessWidget {
  final Widget child;

  const ProfilePadding({super.key, required this.child});
  @override
  Widget build(BuildContext ctx) {
    return Padding(
      padding: EdgeInsetsGeometry.fromLTRB(5, 10, 10, 10),
      child: child,
    );
  }
}

class _LargeScaffold extends StatelessWidget {
  @override
  Widget build(BuildContext ctx) {
    ScreenConfiguration screen = Provider.of<ScreenConfiguration>(ctx);
    List<Widget> leftChildren = [
      Padding(
        padding: const EdgeInsets.all(15),
        child: UserStepsSliderProvider(),
      ),
    ];
    Widget leftCol = ConstrainedBox(
      constraints: BoxConstraints(
        minWidth: 400,
        maxWidth: 400,
        maxHeight: screen.height,
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.start,
        children: leftChildren,
      ),
    );

    Widget mwrow = Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        //Expanded(child: Container(color: Colors.blue)),
        Expanded(
          child: GraphicsPadding(
            child: TrackView.make(allkinds(), TrackData.map),
          ),
        ),
      ],
    );

    List<Widget> rightChildren = [
      ConstrainedBox(
        constraints: BoxConstraints(minHeight: 275, maxHeight: 275),
        child: ProfilePadding(
          child: TrackView.make(allkinds(), TrackData.profile),
        ),
      ),
      Expanded(child: mwrow),
    ];

    Widget rightCol = Expanded(
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: screen.width - 500),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisAlignment: MainAxisAlignment.end,
          children: rightChildren,
        ),
      ),
    );

    Widget div = VerticalDivider(
      color: Colors.lightBlue,
      thickness: 1,
      width: 1, // This is the horizontal space the widget occupies
    );
    return Scaffold(
      appBar: AppBar(title: const Text('Overview')),
      body: Row(children: [leftCol, div, rightCol]),
    );
  }
}

class _LargeScreenProviders extends MultiProvider {
  _LargeScreenProviders({required Widget child})
    : super(
        providers: [
          ChangeNotifierProvider(
            create: (_) => TrackViewsSwitch(exposed: TrackViewsSwitch.wmp()),
          ),
        ],
        child: child,
      );
}

class LargeScreen extends StatelessWidget {
  const LargeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    context.watch<SegmentModel>();
    return _LargeScreenProviders(child: _LargeScaffold());
  }
}
