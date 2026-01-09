import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/futurerenderer.dart';
import 'package:ui/src/widgets/future_rendering_widget.dart';

class ProfileConsumer extends StatelessWidget {
  const ProfileConsumer({super.key});

  @override
  Widget build(BuildContext ctx) {
    return Consumer<ProfileRenderer>(
      builder: (context, pRenderer, child) {
        // It would be more accurate to check visibility with a scroll controller
        // at the list view level. Because "Callbacks are not fired immediately
        // on visibility changes."
        developer.log("update profile renderer:${pRenderer.id()}");
        pRenderer.setSize(Size(1000, 285));
        return FutureRenderingWidget(future: pRenderer, interactive: false);
      },
    );
  }
}

class YAxisConsumer extends StatelessWidget {
  const YAxisConsumer({super.key});

  @override
  Widget build(BuildContext context) {
    return Consumer<YAxisRenderer>(
      builder: (context, yRenderer, child) {
        yRenderer.setSize(Size(1000, 285));
        return FutureRenderingWidget(future: yRenderer, interactive: false);
      },
    );
  }
}

class ProfileScrollWidget extends StatelessWidget {
  const ProfileScrollWidget({super.key});

  @override
  Widget build(BuildContext context) {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: ProfileConsumer(),
    );
  }
}

class ProfileStack extends StatelessWidget {
  final double profileHeight;
  const ProfileStack({super.key, required this.profileHeight});

  @override
  Widget build(BuildContext context) {
    developer.log("profile-Height=$profileHeight");
    return LayoutBuilder(
      builder: (context, constraints) {
        var scrollView = ProfileScrollWidget();
        var box = ConstrainedBox(
          constraints: BoxConstraints(maxHeight: profileHeight),
          child: Stack(
            children: [
              Positioned.fill(child: scrollView),
              if (constraints.maxWidth < 1000)
                Positioned(
                  left: 0,
                  top: 0,
                  bottom: 0,
                  child: SizedBox(width: 50, child: YAxisConsumer()),
                ),
            ],
          ),
        );
        return ConstrainedBox(
          constraints: const BoxConstraints(
            maxWidth: 1000, // Constrain the width to a maximum of 1000 pixels
          ),
          child: box,
        );
      },
    );
  }
}
