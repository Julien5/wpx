import 'dart:developer' as developer;
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/screens/home/home_screen.dart';
import 'package:ui/src/widgets/small.dart';
import 'package:visibility_detector/visibility_detector.dart';

import 'eventwidget.dart';
import 'model.dart';

class GPXStrings {
  final LoadScreenModel screenModel;

  GPXStrings({required this.screenModel});

  bridge.SegmentStatistics? statistics;
  void setData(bridge.SegmentStatistics s) {
    statistics = s;
  }

  String? km() {
    if (statistics == null) {
      return null;
    }
    double km = statistics!.distanceEnd / 1000;
    return "${km.toStringAsFixed(0)} km";
  }

  String? elevation() {
    if (statistics == null) {
      return null;
    }
    double e = statistics!.elevationGain;
    return "${e.toStringAsFixed(0)} m";
  }
}

class GPXCard extends StatelessWidget {
  const GPXCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    GPXStrings strings = GPXStrings(screenModel: model);
    if (model.hasDone(Job.gpx)) {
      strings.setData(model.statistics());
    }
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Track"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Length"),
            ScreenEventWidget(target: Job.gpx, forcedString: strings.km()),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Elevation"),
            ScreenEventWidget(
              target: Job.gpx,
              forcedString: strings.elevation(),
            ),
          ],
        ),
      ],
    );

    return Card(elevation: 4, child: inner);
  }
}

class ControlStrings {
  final LoadScreenModel screenModel;

  ControlStrings({required this.screenModel});

  String? count() {
    if (!screenModel.hasDone(Job.controls)) {
      return null;
    }
    return "${screenModel.controlsCount()}";
  }
}

class ControlsCard extends StatelessWidget {
  const ControlsCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    ControlStrings strings = ControlStrings(screenModel: model);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "Controls"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Number"),
            ScreenEventWidget(
              target: Job.controls,
              forcedString: strings.count(),
            ),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

class OSMCard extends StatelessWidget {
  const OSMCard({super.key});

  @override
  Widget build(BuildContext ctx) {
    developer.log("OSMCard build ");
    LoadScreenModel _ = Provider.of<LoadScreenModel>(ctx);
    Widget inner = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(children: [SmallText(text: "OSM"), SmallText(text: "")]),
        TableRow(
          children: [
            SmallText(text: "Status"),
            ScreenEventWidget(target: Job.osm),
          ],
        ),
      ],
    );
    return Card(elevation: 4, child: inner);
  }
}

String title(LoadScreenModel model) {
  if (model.doneAll()) {
    return "Done";
  }
  return "Loading...";
}

class BodyWidget extends StatelessWidget {
  const BodyWidget({super.key});

  void gotoWheel(BuildContext context) {
    Navigator.of(context).pushNamed(RouteManager.wheelView);
  }

  @override
  Widget build(BuildContext ctx) {
    Widget vspace = SizedBox(height: 20);
    return ConstrainedBox(
      constraints: BoxConstraints(maxWidth: 500),
      child: Center(
        child: Padding(
          padding: const EdgeInsets.all(20),
          child: Column(
            children: [
              GPXCard(),
              vspace,
              ControlsCard(),
              vspace,
              OSMCard(),
              vspace,
              ElevatedButton(
                onPressed: () => {gotoWheel(ctx)},
                child: Text("OK"),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class LoadScreen extends StatelessWidget {
  const LoadScreen({super.key});

  Widget buildScaffold(BuildContext ctx) {
    LoadScreenModel model = Provider.of<LoadScreenModel>(ctx);
    return Scaffold(
      appBar: AppBar(title: Text(title(model))),
      body: BodyWidget(),
    );
  }

  void onVisibilityChanged(
    BuildContext context,
    VisibilityInfo visibilityInfo,
  ) {
    if (!context.mounted) {
      return;
    }
    var visiblePercentage = visibilityInfo.visibleFraction * 100;
    if (visiblePercentage == 100) {
      LoadScreenModel model = Provider.of<LoadScreenModel>(
        context,
        listen: false,
      );
      // The screen is now 100% visible and the transition is done
      if (model.needsStart()) {
        model.start();
      }
    }
  }

  @override
  Widget build(BuildContext ctx) {
    Provider.of<LoadScreenModel>(ctx);
    return VisibilityDetector(
      key: Key('LoadingScreen'),
      onVisibilityChanged: (info) => onVisibilityChanged(ctx, info),
      child: buildScaffold(ctx),
    );
  }
}

class LoadScreenProviders extends MultiProvider {
  final UserInput userInput;
  LoadScreenProviders({
    super.key,
    required RootModel root,
    required this.userInput,
    required Widget child,
  }) : super(
         providers: [
           ChangeNotifierProvider.value(value: root),
           ChangeNotifierProvider.value(value: root.eventModel()),
           ChangeNotifierProxyProvider2<RootModel, EventModel, LoadScreenModel>(
             create: (context) {
               RootModel root = Provider.of<RootModel>(context, listen: false);
               EventModel events = Provider.of<EventModel>(
                 context,
                 listen: false,
               );
               return LoadScreenModel(
                 root: root,
                 events: events,
                 userInput: userInput,
               );
             },
             update: (context, root, event, loadscreen) {
               loadscreen!.onChanged(root, event);
               return loadscreen;
             },
           ),
         ],
         child: child,
       );
}

class LoadProvider extends StatelessWidget {
  final UserInput userInput;
  const LoadProvider({super.key, required this.userInput});

  @override
  Widget build(BuildContext context) {
    RootModel root = Provider.of<RootModel>(context);
    return LoadScreenProviders(
      root: root,
      userInput: userInput,
      child: LoadScreen(),
    );
  }
}
