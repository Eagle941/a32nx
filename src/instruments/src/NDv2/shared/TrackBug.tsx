import { DisplayComponent, EventBus, FSComponent, Subject, Subscribable, VNode } from 'msfssdk';
import { Arinc429Word } from '@shared/arinc429';
import React from 'react';
import { NDSimvars } from '../NDSimvarPublisher';
import { getSmallestAngle } from '../../PFD/PFDUtils';

export interface TrackBugProps {
    bus: EventBus,
    isUsingTrackUpMode: Subscribable<boolean>,
}

export class TrackBug extends DisplayComponent<TrackBugProps> {
    private readonly bugRef = FSComponent.createRef<SVGGElement>();

    private readonly diffSubject = Subject.create(0);

    private readonly headingWord = Subject.create<Arinc429Word>(Arinc429Word.empty());

    private readonly trackWord = Subject.create<Arinc429Word>(Arinc429Word.empty());

    onAfterRender(node: VNode) {
        super.onAfterRender(node);

        const sub = this.props.bus.getSubscriber<NDSimvars>();

        sub.on('heading').whenChanged().handle((v) => {
            this.headingWord.set(new Arinc429Word(v));
            this.handleDisplay();
        });

        sub.on('groundTrack').whenChanged().handle((v) => {
            this.trackWord.set(new Arinc429Word(v));
            this.handleDisplay();
        });
    }

    private handleDisplay() {
        const headingValid = this.headingWord.get().isNormalOperation();
        const trackValid = this.trackWord.get().isNormalOperation();

        if (headingValid && trackValid) {
            let diff;
            if (this.props.isUsingTrackUpMode.get()) {
                diff = 0;
            } else {
                diff = getSmallestAngle(this.trackWord.get().value, this.headingWord.get().value);
            }

            this.diffSubject.set(diff);

            if (diff <= 40) {
                this.bugRef.instance.style.visibility = 'visible';
            } else {
                this.bugRef.instance.style.visibility = 'hidden';
            }
        } else {
            this.bugRef.instance.style.visibility = 'hidden';
        }
    }

    render(): VNode | null {
        return (
            <g ref={this.bugRef} transform={this.diffSubject.map((diff) => `rotate(${diff} 384 620)`)}>
                <path
                    d="M384,128 L378,138 L384,148 L390,138 L384,128"
                    class="shadow rounded"
                    strokeWidth={4.5}
                />
                <path
                    d="M384,128 L378,138 L384,148 L390,138 L384,128"
                    class="Green rounded"
                    strokeWidth={3}
                />
            </g>
        );
    }
}