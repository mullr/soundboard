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
    constructor() {
        super();
        this.state = { collections: [], playing: [] };
    }

    componentDidMount() {
        fetch('/collection')
            .then((response) => response.json())
            .then((data) => {
                this.setState({ collections: data, playing: this.state.playing });
            });
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
                                                 clips: coll.clips })))));
    }

    stop_all() {
        stop_all_request()
    }
}

class Collection extends Component {
    constructor() {
        super();
        this.props = { id: null, name: "", clips: []};
    }

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
                                      name: clip.name }))))));
    }

    play_random() {
        let random_clip = this.props.clips[Math.floor(Math.random()*this.props.clips.length)];
        play_request(this.props.id, random_clip.id);
    }
}

class Clip extends Component {
    constructor() {
        super();
        this.props = { coll_id: null, id: null, name: "" };
        this.state  = { playing: true };
    }

    render() {
        return el('div.card', { id: `clip-${this.props.id}` },
                  e('div.card-body',
                    el('a', { href: '#', onClick: () => this.play() },
                       this.props.name ),
                    this.props.playing ? e('span', ' ▶') : null
                   ));
    }

    play() {
        play_request(this.props.coll_id, this.props.id)
    }
}

render(h(App), document.body);

function play_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/play`, { method: 'POST' });
}

function stop_all_request() {
    fetch('/stop_all', { method: 'POST' });
}
