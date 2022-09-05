import '/preact/debug.mjs';
import { h, Component, Fragment, render, createContext } from '/preact/preact.mjs';
import { useState, useEffect, useContext } from '/preact/hooks.mjs';

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

class EventBus {
    constructor() { this.bus = {}; }
    off(id) { delete this.bus[id]; }
    on(id, callback) { this.bus[id] = callback; }
    emit(id, ...params) { if(this.bus[id]) { this.bus[id](...params); } }
}

const Bus = createContext(new EventBus());

const KindDisplayName = {
    "Drops": "Drops",
    "BackgroundMusic": "Background Music",
    "BattleMusic": "Battle Music",
    "Fx": "FX",
    "Ambience": "Ambience",
};

function App(props) {
    const [collections, setCollections] = useState([]);
    const bus = useContext(Bus);

    const on_backend_message = (sse_event) => {
        let event = JSON.parse(sse_event.data);
        if (event.Started !== undefined) {
            bus.emit(`${event.Started.coll_id}/${event.Started.clip_id}`, {event: "Started", duration: event.Started.duration});
        } else if (event.Stopped !== undefined) {
            bus.emit(`${event.Stopped.coll_id}/${event.Stopped.clip_id}`, {event: "Stopped"});
        }
    };

    // init effects
    useEffect(() => {
        fetch('/collection')
            .then((response) => response.json())
            .then((data) => setCollections(data));

        const event_source = new EventSource("/events");
        event_source.onmessage = on_backend_message;

        return () => { event_source.close() };
    }, []);
    
    const stop_all = (e) => {
        stop_all_request();
        e.preventDefault();
    };

    return e('div.container',
             e('header',
               e('span.fs-1.me-3', "The Soundboard"),
               e('big', e('b', el('a', { href: '#', onClick: stop_all },
                                  "STOP ALL")))),
             e('main',
               collections.map(
                   coll => h(Fragment, null,
                             el('hr'),
                             h(Collection, { id: coll.id,
                                             name: coll.name,
                                             clips: coll.clips,
                                             kind: KindDisplayName[coll.kind]})))));
}

function Collection(props) {
    let chunks = [];
    for (let i = 0; i < props.clips.length; i += 3) {
        chunks.push(props.clips.slice(i, i + 3));
    }

    let play_random = (e) => {
        let random_clip = props.clips[Math.floor(Math.random()*props.clips.length)];
        play_clip_request(props.id, random_clip.id);
        e.preventDefault();
    };

    return el('div.d-grid.gap-3', { key: `coll-${props.id}` },
              e('div.row',
                e('div.col',
                  e('span.fs-2.me-3', props.name),
                  e('span.badge.rounded-pill.text-bg-primary.me-3', props.kind),
                  el('a', { href: "#", onClick: play_random }, "Play Random"))),
              chunks.map(chunk =>
                  e('div.row', chunk.map(clip =>
                      e('div.col-md-4',
                        h(Clip, { coll_id: props.id,
                                  id: clip.id,
                                  name: clip.name}))))));
}

function Clip(props) {
    const play = (e) => {
        play_clip_request(props.coll_id, props.id);
        e.preventDefault();
    };

    const stop = (e) => {
        stop_clip_request(props.coll_id, props.id);
        e.preventDefault();
    };

    const [playing, setPlaying] = useState(false);

    const on_message = (message) => {
        switch (message.event) {
        case "Started":
            setPlaying(true);
            break;
        case "Stopped":
            setPlaying(false);
            break;
        }
    };

    const bus = useContext(Bus);
    useEffect(() => {
        let key = `${props.coll_id}/${props.id}`;
        bus.on(key, on_message);
        return () => bus.off(key);
    }, []);

    return el('div.card', { key: `clip-${props.id}` },
              e('div.card-body',
                el('a', { href: '#', onClick: play }, props.name ),
                e('span', ' '),
                playing ? el('a', { href: '#', onClick: stop }, 'X') : null));
}

render(h(App), document.body);

function play_clip_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/play`, { method: 'POST' });
}

function stop_clip_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/stop`, { method: 'POST' });
}

function stop_coll_request(coll_id, clip_id) {
    fetch(`/collection/${coll_id}/clip/${clip_id}/stop`, { method: 'POST' });
}

function stop_all_request() {
    fetch('/stop_all', { method: 'POST' });
}
