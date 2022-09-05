import { h, Component, Fragment, render } from 'https://unpkg.com/preact?module';

// like h(), but with tag.class.class syntax
function el(tag, props, ...args) {
    let parts = tag.split('.');
    tag = parts.shift();
    props = props || {};
    props['class'] = (props['class'] || '') + parts.join(' ')
    return h(tag, props, args);
}

// in case you have no props
function e(tag, ...args) {
    return el(tag, null, args);
}


class App extends Component {
    state = { collections: [], playing: {} };

    componentDidMount() {
        fetch('/collection')
            .then((response) => response.json())
            .then((data) => {
                this.setState({ collections: data, playing: this.state.playing });
            });

        const event_source = new EventSource("/events");
        event_source.onmessage = (event) => {
            let data = JSON.parse(event.data);
            data.forEach(event => {
                if (event.Started) {
                    this.playback_started(event.Started.coll_id, event.Started.clip_id, event.Started.duration);
                } else if (event.Stopped) {
                    this.playback_stopped(event.Stopped.coll_id, event.Stopped.clip_id);
                }
            })
        };
    }
    
    render() {
        return e('div.container', 
                 e('header',
                   e('span.fs-1.me-3', "The Soundboard"),
                   e('big', e('b', el('a', { href: '#',
                                             onClick: () => this.stop_all() },
                                      "STOP ALL")))),
                 e('main', 
                   this.state.collections.map(
                       coll => h(Fragment, null,
                                 el('hr'),
                                 h(Collection, { id: coll.id,
                                                 name: coll.name,
                                                 clips: coll.clips,
                                                 playing: this.state.playing[coll.id] || {} })))));
    }

    stop_all() {
        stop_all_request()
    }

    playback_started(coll_id, clip_id, duration) {
        let state = this.state;
        if (!state.playing[coll_id]) {
            state.playing[coll_id] = {};
        }
        state.playing[coll_id][clip_id] = true;
        this.setState(state);
        console.log(this.state.playing);
    }

    playback_stopped(coll_id, clip_id) {
        let state = this.state;
        if (state.playing[coll_id]) {
            state.playing[coll_id][clip_id] = false;
        }

        this.setState(state);
        console.log(this.state.playing);
    }
}

class Collection extends Component {
    props = { id: null, name: "", clips: [], playing: {}};

    render() {
        let chunks = [];
        for (let i = 0; i < this.props.clips.length; i += 3) {
            chunks.push(this.props.clips.slice(i, i + 3));
        }

        return el('div.d-grid.gap-3', { id: `coll-${this.props.id}` },
                  e('div.row',
                    e('div.col',
                      e('span.fs-2.me-3', this.props.name),
                      el('a', { href: "#", onClick: () => this.play_random() }, "Random"))),
                  chunks.map(chunk =>
                      e('div.row', chunk.map(clip =>
                          e('div.col-md-4', {'class': 'col-md-4'}, 
                            h(Clip, { coll_id: this.props.id,
                                      id: clip.id,
                                      name: clip.name,
                                      playing: this.props.playing[clip.id]?true:false}))))));
    }

    play_random() {
        let random_clip = this.props.clips[Math.floor(Math.random()*this.props.clips.length)];
        play_request(this.props.id, random_clip.id);
    }
}

class Clip extends Component {
    props = { coll_id: null, id: null, name: "", playing: false };

    render() {
        return el('div.card', { id: `clip-${this.props.id}` },
                  e('div.card-body',
                    el('a', { href: '#', onClick: () => this.play() },
                       this.props.name ),

                    e('span', ' '),
                    this.props.playing ? el('a', { href: '#', onClick: () => this.stop() }, 'X') : null
                   ));
    }

    play() {
        play_request(this.props.coll_id, this.props.id)
    }

    stop() {
        stop_request(this.props.coll_id, this.props.id)
    }

}

render(h(App), document.body);

function play_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/play`, { method: 'POST' });
}

function stop_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/stop`, { method: 'POST' });
}

function stop_all_request() {
    fetch('/stop_all', { method: 'POST' });
}
