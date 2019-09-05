
(function(l, i, v, e) { v = l.createElement(i); v.async = 1; v.src = '//' + (location.host || 'localhost').split(':')[0] + ':35729/livereload.js?snipver=1'; e = l.getElementsByTagName(i)[0]; e.parentNode.insertBefore(v, e)})(document, 'script');
var app = (function () {
    'use strict';

    function noop() { }
    function add_location(element, file, line, column, char) {
        element.__svelte_meta = {
            loc: { file, line, column, char }
        };
    }
    function run(fn) {
        return fn();
    }
    function blank_object() {
        return Object.create(null);
    }
    function run_all(fns) {
        fns.forEach(run);
    }
    function is_function(thing) {
        return typeof thing === 'function';
    }
    function safe_not_equal(a, b) {
        return a != a ? b == b : a !== b || ((a && typeof a === 'object') || typeof a === 'function');
    }
    function validate_store(store, name) {
        if (!store || typeof store.subscribe !== 'function') {
            throw new Error(`'${name}' is not a store with a 'subscribe' method`);
        }
    }
    function subscribe(store, callback) {
        const unsub = store.subscribe(callback);
        return unsub.unsubscribe ? () => unsub.unsubscribe() : unsub;
    }
    function component_subscribe(component, store, callback) {
        component.$$.on_destroy.push(subscribe(store, callback));
    }

    function append(target, node) {
        target.appendChild(node);
    }
    function insert(target, node, anchor) {
        target.insertBefore(node, anchor || null);
    }
    function detach(node) {
        node.parentNode.removeChild(node);
    }
    function element(name) {
        return document.createElement(name);
    }
    function text(data) {
        return document.createTextNode(data);
    }
    function space() {
        return text(' ');
    }
    function empty() {
        return text('');
    }
    function listen(node, event, handler, options) {
        node.addEventListener(event, handler, options);
        return () => node.removeEventListener(event, handler, options);
    }
    function attr(node, attribute, value) {
        if (value == null)
            node.removeAttribute(attribute);
        else
            node.setAttribute(attribute, value);
    }
    function to_number(value) {
        return value === '' ? undefined : +value;
    }
    function children(element) {
        return Array.from(element.childNodes);
    }
    function set_data(text, data) {
        data = '' + data;
        if (text.data !== data)
            text.data = data;
    }
    function set_input_value(input, value) {
        if (value != null || input.value) {
            input.value = value;
        }
    }
    function custom_event(type, detail) {
        const e = document.createEvent('CustomEvent');
        e.initCustomEvent(type, false, false, detail);
        return e;
    }

    let current_component;
    function set_current_component(component) {
        current_component = component;
    }
    function createEventDispatcher() {
        const component = current_component;
        return (type, detail) => {
            const callbacks = component.$$.callbacks[type];
            if (callbacks) {
                // TODO are there situations where events could be dispatched
                // in a server (non-DOM) environment?
                const event = custom_event(type, detail);
                callbacks.slice().forEach(fn => {
                    fn.call(component, event);
                });
            }
        };
    }

    const dirty_components = [];
    const binding_callbacks = [];
    const render_callbacks = [];
    const flush_callbacks = [];
    const resolved_promise = Promise.resolve();
    let update_scheduled = false;
    function schedule_update() {
        if (!update_scheduled) {
            update_scheduled = true;
            resolved_promise.then(flush);
        }
    }
    function add_render_callback(fn) {
        render_callbacks.push(fn);
    }
    function flush() {
        const seen_callbacks = new Set();
        do {
            // first, call beforeUpdate functions
            // and update components
            while (dirty_components.length) {
                const component = dirty_components.shift();
                set_current_component(component);
                update(component.$$);
            }
            while (binding_callbacks.length)
                binding_callbacks.pop()();
            // then, once components are updated, call
            // afterUpdate functions. This may cause
            // subsequent updates...
            for (let i = 0; i < render_callbacks.length; i += 1) {
                const callback = render_callbacks[i];
                if (!seen_callbacks.has(callback)) {
                    callback();
                    // ...so guard against infinite loops
                    seen_callbacks.add(callback);
                }
            }
            render_callbacks.length = 0;
        } while (dirty_components.length);
        while (flush_callbacks.length) {
            flush_callbacks.pop()();
        }
        update_scheduled = false;
    }
    function update($$) {
        if ($$.fragment) {
            $$.update($$.dirty);
            run_all($$.before_update);
            $$.fragment.p($$.dirty, $$.ctx);
            $$.dirty = null;
            $$.after_update.forEach(add_render_callback);
        }
    }
    const outroing = new Set();
    let outros;
    function group_outros() {
        outros = {
            r: 0,
            c: [],
            p: outros // parent group
        };
    }
    function check_outros() {
        if (!outros.r) {
            run_all(outros.c);
        }
        outros = outros.p;
    }
    function transition_in(block, local) {
        if (block && block.i) {
            outroing.delete(block);
            block.i(local);
        }
    }
    function transition_out(block, local, detach, callback) {
        if (block && block.o) {
            if (outroing.has(block))
                return;
            outroing.add(block);
            outros.c.push(() => {
                outroing.delete(block);
                if (callback) {
                    if (detach)
                        block.d(1);
                    callback();
                }
            });
            block.o(local);
        }
    }

    function destroy_block(block, lookup) {
        block.d(1);
        lookup.delete(block.key);
    }
    function update_keyed_each(old_blocks, changed, get_key, dynamic, ctx, list, lookup, node, destroy, create_each_block, next, get_context) {
        let o = old_blocks.length;
        let n = list.length;
        let i = o;
        const old_indexes = {};
        while (i--)
            old_indexes[old_blocks[i].key] = i;
        const new_blocks = [];
        const new_lookup = new Map();
        const deltas = new Map();
        i = n;
        while (i--) {
            const child_ctx = get_context(ctx, list, i);
            const key = get_key(child_ctx);
            let block = lookup.get(key);
            if (!block) {
                block = create_each_block(key, child_ctx);
                block.c();
            }
            else if (dynamic) {
                block.p(changed, child_ctx);
            }
            new_lookup.set(key, new_blocks[i] = block);
            if (key in old_indexes)
                deltas.set(key, Math.abs(i - old_indexes[key]));
        }
        const will_move = new Set();
        const did_move = new Set();
        function insert(block) {
            transition_in(block, 1);
            block.m(node, next);
            lookup.set(block.key, block);
            next = block.first;
            n--;
        }
        while (o && n) {
            const new_block = new_blocks[n - 1];
            const old_block = old_blocks[o - 1];
            const new_key = new_block.key;
            const old_key = old_block.key;
            if (new_block === old_block) {
                // do nothing
                next = new_block.first;
                o--;
                n--;
            }
            else if (!new_lookup.has(old_key)) {
                // remove old block
                destroy(old_block, lookup);
                o--;
            }
            else if (!lookup.has(new_key) || will_move.has(new_key)) {
                insert(new_block);
            }
            else if (did_move.has(old_key)) {
                o--;
            }
            else if (deltas.get(new_key) > deltas.get(old_key)) {
                did_move.add(new_key);
                insert(new_block);
            }
            else {
                will_move.add(old_key);
                o--;
            }
        }
        while (o--) {
            const old_block = old_blocks[o];
            if (!new_lookup.has(old_block.key))
                destroy(old_block, lookup);
        }
        while (n)
            insert(new_blocks[n - 1]);
        return new_blocks;
    }
    function mount_component(component, target, anchor) {
        const { fragment, on_mount, on_destroy, after_update } = component.$$;
        fragment.m(target, anchor);
        // onMount happens before the initial afterUpdate
        add_render_callback(() => {
            const new_on_destroy = on_mount.map(run).filter(is_function);
            if (on_destroy) {
                on_destroy.push(...new_on_destroy);
            }
            else {
                // Edge case - component was destroyed immediately,
                // most likely as a result of a binding initialising
                run_all(new_on_destroy);
            }
            component.$$.on_mount = [];
        });
        after_update.forEach(add_render_callback);
    }
    function destroy_component(component, detaching) {
        if (component.$$.fragment) {
            run_all(component.$$.on_destroy);
            component.$$.fragment.d(detaching);
            // TODO null out other refs, including component.$$ (but need to
            // preserve final state?)
            component.$$.on_destroy = component.$$.fragment = null;
            component.$$.ctx = {};
        }
    }
    function make_dirty(component, key) {
        if (!component.$$.dirty) {
            dirty_components.push(component);
            schedule_update();
            component.$$.dirty = blank_object();
        }
        component.$$.dirty[key] = true;
    }
    function init(component, options, instance, create_fragment, not_equal, prop_names) {
        const parent_component = current_component;
        set_current_component(component);
        const props = options.props || {};
        const $$ = component.$$ = {
            fragment: null,
            ctx: null,
            // state
            props: prop_names,
            update: noop,
            not_equal,
            bound: blank_object(),
            // lifecycle
            on_mount: [],
            on_destroy: [],
            before_update: [],
            after_update: [],
            context: new Map(parent_component ? parent_component.$$.context : []),
            // everything else
            callbacks: blank_object(),
            dirty: null
        };
        let ready = false;
        $$.ctx = instance
            ? instance(component, props, (key, value) => {
                if ($$.ctx && not_equal($$.ctx[key], $$.ctx[key] = value)) {
                    if ($$.bound[key])
                        $$.bound[key](value);
                    if (ready)
                        make_dirty(component, key);
                }
            })
            : props;
        $$.update();
        ready = true;
        run_all($$.before_update);
        $$.fragment = create_fragment($$.ctx);
        if (options.target) {
            if (options.hydrate) {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment.l(children(options.target));
            }
            else {
                // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
                $$.fragment.c();
            }
            if (options.intro)
                transition_in(component.$$.fragment);
            mount_component(component, options.target, options.anchor);
            flush();
        }
        set_current_component(parent_component);
    }
    class SvelteComponent {
        $destroy() {
            destroy_component(this, 1);
            this.$destroy = noop;
        }
        $on(type, callback) {
            const callbacks = (this.$$.callbacks[type] || (this.$$.callbacks[type] = []));
            callbacks.push(callback);
            return () => {
                const index = callbacks.indexOf(callback);
                if (index !== -1)
                    callbacks.splice(index, 1);
            };
        }
        $set() {
            // overridden by instance, if it has props
        }
    }
    class SvelteComponentDev extends SvelteComponent {
        constructor(options) {
            if (!options || (!options.target && !options.$$inline)) {
                throw new Error(`'target' is a required option`);
            }
            super();
        }
        $destroy() {
            super.$destroy();
            this.$destroy = () => {
                console.warn(`Component was already destroyed`); // eslint-disable-line no-console
            };
        }
    }

    /* src/Login.svelte generated by Svelte v3.9.1 */

    const file = "src/Login.svelte";

    function create_fragment(ctx) {
    	var h1, t0, t1, form, label0, t3, input0, t4, label1, t6, input1, t7, button, t8, form_action_value;

    	return {
    		c: function create() {
    			h1 = element("h1");
    			t0 = text(ctx.prettyType);
    			t1 = space();
    			form = element("form");
    			label0 = element("label");
    			label0.textContent = "Username";
    			t3 = space();
    			input0 = element("input");
    			t4 = space();
    			label1 = element("label");
    			label1.textContent = "Password";
    			t6 = space();
    			input1 = element("input");
    			t7 = space();
    			button = element("button");
    			t8 = text(ctx.prettyType);
    			add_location(h1, file, 5, 0, 111);
    			attr(label0, "for", "username");
    			attr(label0, "class", "svelte-d00odx");
    			add_location(label0, file, 8, 4, 181);
    			attr(input0, "id", "username");
    			attr(input0, "name", "username");
    			add_location(input0, file, 9, 4, 222);
    			attr(label1, "for", "password");
    			attr(label1, "class", "svelte-d00odx");
    			add_location(label1, file, 11, 4, 261);
    			attr(input1, "id", "password");
    			attr(input1, "name", "password");
    			attr(input1, "type", "password");
    			add_location(input1, file, 12, 4, 302);
    			attr(button, "type", "submit");
    			attr(button, "class", "button svelte-d00odx");
    			add_location(button, file, 14, 4, 355);
    			attr(form, "action", form_action_value = "/user/" + ctx.type);
    			attr(form, "method", "POST");
    			add_location(form, file, 6, 0, 133);
    		},

    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, h1, anchor);
    			append(h1, t0);
    			insert(target, t1, anchor);
    			insert(target, form, anchor);
    			append(form, label0);
    			append(form, t3);
    			append(form, input0);
    			append(form, t4);
    			append(form, label1);
    			append(form, t6);
    			append(form, input1);
    			append(form, t7);
    			append(form, button);
    			append(button, t8);
    		},

    		p: function update(changed, ctx) {
    			if ((changed.type) && form_action_value !== (form_action_value = "/user/" + ctx.type)) {
    				attr(form, "action", form_action_value);
    			}
    		},

    		i: noop,
    		o: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(h1);
    				detach(t1);
    				detach(form);
    			}
    		}
    	};
    }

    function instance($$self, $$props, $$invalidate) {
    	let { type = 'login' } = $$props;
        let prettyType = type[0].toUpperCase() + type.slice(1);

    	const writable_props = ['type'];
    	Object.keys($$props).forEach(key => {
    		if (!writable_props.includes(key) && !key.startsWith('$$')) console.warn(`<Login> was created with unknown prop '${key}'`);
    	});

    	$$self.$set = $$props => {
    		if ('type' in $$props) $$invalidate('type', type = $$props.type);
    	};

    	return { type, prettyType };
    }

    class Login extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance, create_fragment, safe_not_equal, ["type"]);
    	}

    	get type() {
    		throw new Error("<Login>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set type(value) {
    		throw new Error("<Login>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    const subscriber_queue = [];
    /**
     * Creates a `Readable` store that allows reading by subscription.
     * @param value initial value
     * @param {StartStopNotifier}start start and stop notifications for subscriptions
     */
    function readable(value, start) {
        return {
            subscribe: writable(value, start).subscribe,
        };
    }
    /**
     * Create a `Writable` store that allows both updating and reading by subscription.
     * @param {*=}value initial value
     * @param {StartStopNotifier=}start start and stop notifications for subscriptions
     */
    function writable(value, start = noop) {
        let stop;
        const subscribers = [];
        function set(new_value) {
            if (safe_not_equal(value, new_value)) {
                value = new_value;
                if (stop) { // store is ready
                    const run_queue = !subscriber_queue.length;
                    for (let i = 0; i < subscribers.length; i += 1) {
                        const s = subscribers[i];
                        s[1]();
                        subscriber_queue.push(s, value);
                    }
                    if (run_queue) {
                        for (let i = 0; i < subscriber_queue.length; i += 2) {
                            subscriber_queue[i][0](subscriber_queue[i + 1]);
                        }
                        subscriber_queue.length = 0;
                    }
                }
            }
        }
        function update(fn) {
            set(fn(value));
        }
        function subscribe(run, invalidate = noop) {
            const subscriber = [run, invalidate];
            subscribers.push(subscriber);
            if (subscribers.length === 1) {
                stop = start(set) || noop;
            }
            run(value);
            return () => {
                const index = subscribers.indexOf(subscriber);
                if (index !== -1) {
                    subscribers.splice(index, 1);
                }
                if (subscribers.length === 0) {
                    stop();
                    stop = null;
                }
            };
        }
        return { set, update, subscribe };
    }

    const BASE_URL = "/document";

    // "pre-fetch"
    const _initial_documents = fetch(BASE_URL).then(response => {
        if (response.status !== 200) {
            return [];
        }
        return response.json();
    });

    const { set, subscribe: subscribe$1, update: update$1 } = writable([], set => _initial_documents.then(set));

    async function create(data) {
        let response = await fetch(BASE_URL, {
            method: 'POST',
            headers: { "content-type": "application/json" },
            body: JSON.stringify(data)
        });

        if (response.status !== 201) {
            let msg = await response.text();
            throw new Error(`HTTP status ${response.status}, message '${msg}'`);
        }
        let id = await response.text();
        update$1(documents => [...documents, { id, data }]);
    }

    async function delete_(id) {
        let response = await fetch(`${BASE_URL}/${id}`, {
            method: "DELETE"
        });
        if (response.status !== 200) {
            let msg = await response.text();
            throw new Error(`HTTP status ${response.status}, message '${msg}'`);
        }
        update$1(documents => {
            const i = documents.findIndex(o => o.id === id);

            if (i === -1) {
                throw new Error(`Expected to find id ${id}, but did not.`)
            }

            return [
                ...documents.slice(0, i),
                ...documents.slice(i + 1)
            ]
        });
    }

    async function put(document) {
        let { id, data } = document;

        let response = await fetch(`${BASE_URL}/${id}`, {
            method: 'PUT',
            headers: { "content-type": "application/json" },
            body: JSON.stringify(data)
        });

        if (response.status !== 200) {
            let msg = await response.text();
            throw new Error(`HTTP status ${response.status}, message '${msg}'`);
        }
        update$1(documents => {
            const i = documents.findIndex(o => o.id === id);

            if (i === -1) {
              throw new Error(`Expected to find id ${id}, but did not.`)
            }

            return [
                ...documents.slice(0, i),
                document,
                ...documents.slice(i + 1)
            ]
        });
    }

    const documents = { create, delete_, subscribe: subscribe$1, put };

    /* src/InputForm.svelte generated by Svelte v3.9.1 */

    const file$1 = "src/InputForm.svelte";

    function create_fragment$1(ctx) {
    	var h2, t0, t1, t2, form, label0, t4, input0, t5, label1, t7, input1, t8, label2, t10, input2, t11, label3, t13, input3, t14, div, button0, t16, button1, dispose;

    	return {
    		c: function create() {
    			h2 = element("h2");
    			t0 = text(ctx.title);
    			t1 = text(" document");
    			t2 = space();
    			form = element("form");
    			label0 = element("label");
    			label0.textContent = "Date";
    			t4 = space();
    			input0 = element("input");
    			t5 = space();
    			label1 = element("label");
    			label1.textContent = "Description";
    			t7 = space();
    			input1 = element("input");
    			t8 = space();
    			label2 = element("label");
    			label2.textContent = "Amount";
    			t10 = space();
    			input2 = element("input");
    			t11 = space();
    			label3 = element("label");
    			label3.textContent = "Done";
    			t13 = space();
    			input3 = element("input");
    			t14 = space();
    			div = element("div");
    			button0 = element("button");
    			button0.textContent = "Save";
    			t16 = space();
    			button1 = element("button");
    			button1.textContent = "Cancel";
    			attr(h2, "class", "subtitle");
    			add_location(h2, file$1, 22, 0, 548);
    			attr(label0, "for", "date");
    			attr(label0, "class", "svelte-belg7s");
    			add_location(label0, file$1, 24, 4, 621);
    			attr(input0, "id", "date");
    			attr(input0, "type", "date");
    			add_location(input0, file$1, 25, 4, 654);
    			attr(label1, "for", "description");
    			attr(label1, "class", "svelte-belg7s");
    			add_location(label1, file$1, 27, 4, 708);
    			attr(input1, "id", "description");
    			attr(input1, "type", "text");
    			add_location(input1, file$1, 28, 4, 755);
    			attr(label2, "for", "amount");
    			attr(label2, "class", "svelte-belg7s");
    			add_location(label2, file$1, 30, 4, 823);
    			attr(input2, "id", "amount");
    			attr(input2, "type", "number");
    			add_location(input2, file$1, 31, 4, 860);
    			attr(label3, "for", "done");
    			attr(label3, "class", "svelte-belg7s");
    			add_location(label3, file$1, 33, 4, 920);
    			attr(input3, "id", "done");
    			attr(input3, "type", "checkbox");
    			add_location(input3, file$1, 34, 4, 953);
    			attr(button0, "type", "submit");
    			add_location(button0, file$1, 37, 8, 1040);
    			add_location(button1, file$1, 38, 8, 1082);
    			attr(div, "class", "action svelte-belg7s");
    			add_location(div, file$1, 36, 4, 1013);
    			add_location(form, file$1, 23, 0, 591);

    			dispose = [
    				listen(input0, "input", ctx.input0_input_handler),
    				listen(input1, "input", ctx.input1_input_handler),
    				listen(input2, "input", ctx.input2_input_handler),
    				listen(input3, "change", ctx.input3_change_handler),
    				listen(button1, "click", ctx.click_handler),
    				listen(form, "submit", ctx.submit)
    			];
    		},

    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, h2, anchor);
    			append(h2, t0);
    			append(h2, t1);
    			insert(target, t2, anchor);
    			insert(target, form, anchor);
    			append(form, label0);
    			append(form, t4);
    			append(form, input0);

    			set_input_value(input0, ctx.data.date);

    			append(form, t5);
    			append(form, label1);
    			append(form, t7);
    			append(form, input1);

    			set_input_value(input1, ctx.data.description);

    			append(form, t8);
    			append(form, label2);
    			append(form, t10);
    			append(form, input2);

    			set_input_value(input2, ctx.data.amount);

    			append(form, t11);
    			append(form, label3);
    			append(form, t13);
    			append(form, input3);

    			input3.checked = ctx.data.done;

    			append(form, t14);
    			append(form, div);
    			append(div, button0);
    			append(div, t16);
    			append(div, button1);
    		},

    		p: function update(changed, ctx) {
    			if (changed.title) {
    				set_data(t0, ctx.title);
    			}

    			if (changed.data) set_input_value(input0, ctx.data.date);
    			if (changed.data && (input1.value !== ctx.data.description)) set_input_value(input1, ctx.data.description);
    			if (changed.data) set_input_value(input2, ctx.data.amount);
    			if (changed.data) input3.checked = ctx.data.done;
    		},

    		i: noop,
    		o: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(h2);
    				detach(t2);
    				detach(form);
    			}

    			run_all(dispose);
    		}
    	};
    }

    function instance$1($$self, $$props, $$invalidate) {
    	

        let { document } = $$props;

        const dispatch = createEventDispatcher();

        function submit(event) {
            event.preventDefault();

            if (isNew) {
                documents.create(data).then(() => dispatch("done"));
            } else {
                documents.put(document).then(() => dispatch("done"));
            }
        }

    	const writable_props = ['document'];
    	Object.keys($$props).forEach(key => {
    		if (!writable_props.includes(key) && !key.startsWith('$$')) console.warn(`<InputForm> was created with unknown prop '${key}'`);
    	});

    	function input0_input_handler() {
    		data.date = this.value;
    		$$invalidate('data', data), $$invalidate('document', document);
    	}

    	function input1_input_handler() {
    		data.description = this.value;
    		$$invalidate('data', data), $$invalidate('document', document);
    	}

    	function input2_input_handler() {
    		data.amount = to_number(this.value);
    		$$invalidate('data', data), $$invalidate('document', document);
    	}

    	function input3_change_handler() {
    		data.done = this.checked;
    		$$invalidate('data', data), $$invalidate('document', document);
    	}

    	function click_handler() {
    		return dispatch("done");
    	}

    	$$self.$set = $$props => {
    		if ('document' in $$props) $$invalidate('document', document = $$props.document);
    	};

    	let data, isNew, title;

    	$$self.$$.update = ($$dirty = { document: 1, isNew: 1 }) => {
    		if ($$dirty.document) { $$invalidate('data', data = document.data); }
    		if ($$dirty.document) { $$invalidate('isNew', isNew = document.id === null); }
    		if ($$dirty.isNew) { $$invalidate('title', title = isNew ? "New" : "Edit"); }
    	};

    	return {
    		document,
    		dispatch,
    		submit,
    		data,
    		title,
    		input0_input_handler,
    		input1_input_handler,
    		input2_input_handler,
    		input3_change_handler,
    		click_handler
    	};
    }

    class InputForm extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$1, create_fragment$1, safe_not_equal, ["document"]);

    		const { ctx } = this.$$;
    		const props = options.props || {};
    		if (ctx.document === undefined && !('document' in props)) {
    			console.warn("<InputForm> was created without expected prop 'document'");
    		}
    	}

    	get document() {
    		throw new Error("<InputForm>: Props cannot be read directly from the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}

    	set document(value) {
    		throw new Error("<InputForm>: Props cannot be set directly on the component instance unless compiling with 'accessors: true' or '<svelte:options accessors/>'");
    	}
    }

    /* src/CRUD.svelte generated by Svelte v3.9.1 */

    const file$2 = "src/CRUD.svelte";

    function get_each_context(ctx, list, i) {
    	const child_ctx = Object.create(ctx);
    	child_ctx.doc = list[i];
    	return child_ctx;
    }

    // (38:0) {#if $documents.length}
    function create_if_block_1(ctx) {
    	var table, thead, tr, th0, t1, th1, t3, th2, t5, th3, t7, th4, t9, each_blocks = [], each_1_lookup = new Map();

    	var each_value = ctx.$documents;

    	const get_key = ctx => ctx.doc.id;

    	for (var i = 0; i < each_value.length; i += 1) {
    		let child_ctx = get_each_context(ctx, each_value, i);
    		let key = get_key(child_ctx);
    		each_1_lookup.set(key, each_blocks[i] = create_each_block(key, child_ctx));
    	}

    	return {
    		c: function create_1() {
    			table = element("table");
    			thead = element("thead");
    			tr = element("tr");
    			th0 = element("th");
    			th0.textContent = "date";
    			t1 = space();
    			th1 = element("th");
    			th1.textContent = "description";
    			t3 = space();
    			th2 = element("th");
    			th2.textContent = "amount";
    			t5 = space();
    			th3 = element("th");
    			th3.textContent = "done";
    			t7 = space();
    			th4 = element("th");
    			th4.textContent = "actions";
    			t9 = space();

    			for (i = 0; i < each_blocks.length; i += 1) each_blocks[i].c();
    			attr(th0, "class", "svelte-1fv41r1");
    			add_location(th0, file$2, 42, 16, 931);
    			attr(th1, "class", "svelte-1fv41r1");
    			add_location(th1, file$2, 43, 16, 961);
    			attr(th2, "class", "svelte-1fv41r1");
    			add_location(th2, file$2, 44, 16, 998);
    			attr(th3, "class", "svelte-1fv41r1");
    			add_location(th3, file$2, 45, 16, 1030);
    			attr(th4, "class", "svelte-1fv41r1");
    			add_location(th4, file$2, 46, 16, 1060);
    			add_location(tr, file$2, 40, 12, 873);
    			add_location(thead, file$2, 39, 8, 853);
    			attr(table, "class", "table svelte-1fv41r1");
    			add_location(table, file$2, 38, 4, 825);
    		},

    		m: function mount(target, anchor) {
    			insert(target, table, anchor);
    			append(table, thead);
    			append(thead, tr);
    			append(tr, th0);
    			append(tr, t1);
    			append(tr, th1);
    			append(tr, t3);
    			append(tr, th2);
    			append(tr, t5);
    			append(tr, th3);
    			append(tr, t7);
    			append(tr, th4);
    			append(table, t9);

    			for (i = 0; i < each_blocks.length; i += 1) each_blocks[i].m(table, null);
    		},

    		p: function update(changed, ctx) {
    			const each_value = ctx.$documents;
    			each_blocks = update_keyed_each(each_blocks, changed, get_key, 1, ctx, each_value, each_1_lookup, table, destroy_block, create_each_block, null, get_each_context);
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(table);
    			}

    			for (i = 0; i < each_blocks.length; i += 1) each_blocks[i].d();
    		}
    	};
    }

    // (50:4) {#each $documents as doc (doc.id)}
    function create_each_block(key_1, ctx) {
    	var tr, td0, t0_value = ctx.doc.data.date + "", t0, t1, td1, t2_value = ctx.doc.data.description + "", t2, t3, td2, t4_value = ctx.doc.data.amount + "", t4, t5, td3, t6_value = ctx.doc.data.done ? "✅" : "❌" + "", t6, t7, td4, button0, t9, button1, t11, dispose;

    	function click_handler() {
    		return ctx.click_handler(ctx);
    	}

    	function click_handler_1() {
    		return ctx.click_handler_1(ctx);
    	}

    	return {
    		key: key_1,

    		first: null,

    		c: function create_1() {
    			tr = element("tr");
    			td0 = element("td");
    			t0 = text(t0_value);
    			t1 = space();
    			td1 = element("td");
    			t2 = text(t2_value);
    			t3 = space();
    			td2 = element("td");
    			t4 = text(t4_value);
    			t5 = space();
    			td3 = element("td");
    			t6 = text(t6_value);
    			t7 = space();
    			td4 = element("td");
    			button0 = element("button");
    			button0.textContent = "Edit";
    			t9 = space();
    			button1 = element("button");
    			button1.textContent = "Delete";
    			t11 = space();
    			attr(td0, "class", "svelte-1fv41r1");
    			add_location(td0, file$2, 52, 12, 1215);
    			attr(td1, "class", "svelte-1fv41r1");
    			add_location(td1, file$2, 53, 12, 1252);
    			attr(td2, "class", "svelte-1fv41r1");
    			add_location(td2, file$2, 54, 12, 1296);
    			attr(td3, "class", "svelte-1fv41r1");
    			add_location(td3, file$2, 55, 12, 1335);
    			attr(button0, "class", "svelte-1fv41r1");
    			add_location(button0, file$2, 57, 16, 1405);
    			attr(button1, "class", "svelte-1fv41r1");
    			add_location(button1, file$2, 58, 16, 1470);
    			attr(td4, "class", "svelte-1fv41r1");
    			add_location(td4, file$2, 56, 12, 1384);
    			add_location(tr, file$2, 50, 8, 1159);

    			dispose = [
    				listen(button0, "click", click_handler),
    				listen(button1, "click", click_handler_1)
    			];

    			this.first = tr;
    		},

    		m: function mount(target, anchor) {
    			insert(target, tr, anchor);
    			append(tr, td0);
    			append(td0, t0);
    			append(tr, t1);
    			append(tr, td1);
    			append(td1, t2);
    			append(tr, t3);
    			append(tr, td2);
    			append(td2, t4);
    			append(tr, t5);
    			append(tr, td3);
    			append(td3, t6);
    			append(tr, t7);
    			append(tr, td4);
    			append(td4, button0);
    			append(td4, t9);
    			append(td4, button1);
    			append(tr, t11);
    		},

    		p: function update(changed, new_ctx) {
    			ctx = new_ctx;
    			if ((changed.$documents) && t0_value !== (t0_value = ctx.doc.data.date + "")) {
    				set_data(t0, t0_value);
    			}

    			if ((changed.$documents) && t2_value !== (t2_value = ctx.doc.data.description + "")) {
    				set_data(t2, t2_value);
    			}

    			if ((changed.$documents) && t4_value !== (t4_value = ctx.doc.data.amount + "")) {
    				set_data(t4, t4_value);
    			}

    			if ((changed.$documents) && t6_value !== (t6_value = ctx.doc.data.done ? "✅" : "❌" + "")) {
    				set_data(t6, t6_value);
    			}
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(tr);
    			}

    			run_all(dispose);
    		}
    	};
    }

    // (70:4) {:else}
    function create_else_block(ctx) {
    	var button, dispose;

    	return {
    		c: function create_1() {
    			button = element("button");
    			button.textContent = "Create new document";
    			add_location(button, file$2, 70, 4, 1701);
    			dispose = listen(button, "click", ctx.create);
    		},

    		m: function mount(target, anchor) {
    			insert(target, button, anchor);
    		},

    		p: noop,
    		i: noop,
    		o: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(button);
    			}

    			dispose();
    		}
    	};
    }

    // (65:4) {#if show_input_form}
    function create_if_block(ctx) {
    	var current;

    	var inputform = new InputForm({
    		props: { document: ctx.document },
    		$$inline: true
    	});
    	inputform.$on("done", ctx.done);

    	return {
    		c: function create_1() {
    			inputform.$$.fragment.c();
    		},

    		m: function mount(target, anchor) {
    			mount_component(inputform, target, anchor);
    			current = true;
    		},

    		p: function update(changed, ctx) {
    			var inputform_changes = {};
    			if (changed.document) inputform_changes.document = ctx.document;
    			inputform.$set(inputform_changes);
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(inputform.$$.fragment, local);

    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(inputform.$$.fragment, local);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			destroy_component(inputform, detaching);
    		}
    	};
    }

    function create_fragment$2(ctx) {
    	var h1, t1, t2, current_block_type_index, if_block1, if_block1_anchor, current;

    	var if_block0 = (ctx.$documents.length) && create_if_block_1(ctx);

    	var if_block_creators = [
    		create_if_block,
    		create_else_block
    	];

    	var if_blocks = [];

    	function select_block_type(changed, ctx) {
    		if (ctx.show_input_form) return 0;
    		return 1;
    	}

    	current_block_type_index = select_block_type(null, ctx);
    	if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);

    	return {
    		c: function create_1() {
    			h1 = element("h1");
    			h1.textContent = "Documents";
    			t1 = space();
    			if (if_block0) if_block0.c();
    			t2 = space();
    			if_block1.c();
    			if_block1_anchor = empty();
    			add_location(h1, file$2, 36, 0, 778);
    		},

    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, h1, anchor);
    			insert(target, t1, anchor);
    			if (if_block0) if_block0.m(target, anchor);
    			insert(target, t2, anchor);
    			if_blocks[current_block_type_index].m(target, anchor);
    			insert(target, if_block1_anchor, anchor);
    			current = true;
    		},

    		p: function update(changed, ctx) {
    			if (ctx.$documents.length) {
    				if (if_block0) {
    					if_block0.p(changed, ctx);
    				} else {
    					if_block0 = create_if_block_1(ctx);
    					if_block0.c();
    					if_block0.m(t2.parentNode, t2);
    				}
    			} else if (if_block0) {
    				if_block0.d(1);
    				if_block0 = null;
    			}

    			var previous_block_index = current_block_type_index;
    			current_block_type_index = select_block_type(changed, ctx);
    			if (current_block_type_index === previous_block_index) {
    				if_blocks[current_block_type_index].p(changed, ctx);
    			} else {
    				group_outros();
    				transition_out(if_blocks[previous_block_index], 1, 1, () => {
    					if_blocks[previous_block_index] = null;
    				});
    				check_outros();

    				if_block1 = if_blocks[current_block_type_index];
    				if (!if_block1) {
    					if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
    					if_block1.c();
    				}
    				transition_in(if_block1, 1);
    				if_block1.m(if_block1_anchor.parentNode, if_block1_anchor);
    			}
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(if_block1);
    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(if_block1);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(h1);
    				detach(t1);
    			}

    			if (if_block0) if_block0.d(detaching);

    			if (detaching) {
    				detach(t2);
    			}

    			if_blocks[current_block_type_index].d(detaching);

    			if (detaching) {
    				detach(if_block1_anchor);
    			}
    		}
    	};
    }

    function instance$2($$self, $$props, $$invalidate) {
    	let $documents;

    	validate_store(documents, 'documents');
    	component_subscribe($$self, documents, $$value => { $documents = $$value; $$invalidate('$documents', $documents); });

    	

        let show_input_form = false;
        let document = {};

        function create() {
            $$invalidate('document', document = {
                id: null,
                data: {
                    date: "",
                    description: "",
                    amount: 0,
                    done: false,
                }
            });
            $$invalidate('show_input_form', show_input_form = true);
        }
        function edit(doc) {
            $$invalidate('document', document = doc);
            $$invalidate('show_input_form', show_input_form = true);
        }
        function done() {
            $$invalidate('show_input_form', show_input_form = false);
            $$invalidate('document', document = {});
        }
        function delete_(id) {
            if (document.id === id) {
                $$invalidate('show_input_form', show_input_form = false);
                $$invalidate('document', document = {});
            }
            documents.delete_(id);
        }

    	function click_handler({ doc }) {
    		return edit(doc);
    	}

    	function click_handler_1({ doc }) {
    		return delete_(doc.id);
    	}

    	return {
    		show_input_form,
    		document,
    		create,
    		edit,
    		done,
    		delete_,
    		$documents,
    		click_handler,
    		click_handler_1
    	};
    }

    class CRUD extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$2, create_fragment$2, safe_not_equal, []);
    	}
    }

    // "pre-fetch"
    const _username = fetch('/user').then(r => {
        if (r.status !== 200) {
            return null;
        }
        return r.text();
    });

    const username = readable(null, set => {
        _username.then(set);
    });

    const loggedIn = readable(false, set => {
        _username.then(u => u !== null).then(set);
    });

    /* src/App.svelte generated by Svelte v3.9.1 */

    const file$3 = "src/App.svelte";

    // (30:28) 
    function create_if_block_5(ctx) {
    	var button, dispose;

    	return {
    		c: function create() {
    			button = element("button");
    			button.textContent = "Register";
    			attr(button, "class", "svelte-104w8eq");
    			add_location(button, file$3, 30, 12, 775);
    			dispose = listen(button, "click", ctx.showRegisterDialog);
    		},

    		m: function mount(target, anchor) {
    			insert(target, button, anchor);
    		},

    		p: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(button);
    			}

    			dispose();
    		}
    	};
    }

    // (28:31) 
    function create_if_block_4(ctx) {
    	var button, dispose;

    	return {
    		c: function create() {
    			button = element("button");
    			button.textContent = "Log in";
    			attr(button, "class", "svelte-104w8eq");
    			add_location(button, file$3, 28, 12, 683);
    			dispose = listen(button, "click", ctx.showLoginDialog);
    		},

    		m: function mount(target, anchor) {
    			insert(target, button, anchor);
    		},

    		p: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(button);
    			}

    			dispose();
    		}
    	};
    }

    // (26:8) {#if $loggedIn}
    function create_if_block_3(ctx) {
    	var a, t0, t1, t2;

    	return {
    		c: function create() {
    			a = element("a");
    			t0 = text("Logout (");
    			t1 = text(ctx.$username);
    			t2 = text(")");
    			attr(a, "class", "button svelte-104w8eq");
    			attr(a, "href", "/user/logout");
    			add_location(a, file$3, 26, 12, 578);
    		},

    		m: function mount(target, anchor) {
    			insert(target, a, anchor);
    			append(a, t0);
    			append(a, t1);
    			append(a, t2);
    		},

    		p: function update(changed, ctx) {
    			if (changed.$username) {
    				set_data(t1, ctx.$username);
    			}
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(a);
    			}
    		}
    	};
    }

    // (41:27) 
    function create_if_block_2(ctx) {
    	var current;

    	var login = new Login({
    		props: { type: "register" },
    		$$inline: true
    	});

    	return {
    		c: function create() {
    			login.$$.fragment.c();
    		},

    		m: function mount(target, anchor) {
    			mount_component(login, target, anchor);
    			current = true;
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(login.$$.fragment, local);

    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(login.$$.fragment, local);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			destroy_component(login, detaching);
    		}
    	};
    }

    // (39:24) 
    function create_if_block_1$1(ctx) {
    	var current;

    	var login = new Login({ $$inline: true });

    	return {
    		c: function create() {
    			login.$$.fragment.c();
    		},

    		m: function mount(target, anchor) {
    			mount_component(login, target, anchor);
    			current = true;
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(login.$$.fragment, local);

    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(login.$$.fragment, local);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			destroy_component(login, detaching);
    		}
    	};
    }

    // (37:4) {#if $loggedIn}
    function create_if_block$1(ctx) {
    	var current;

    	var crud = new CRUD({ $$inline: true });

    	return {
    		c: function create() {
    			crud.$$.fragment.c();
    		},

    		m: function mount(target, anchor) {
    			mount_component(crud, target, anchor);
    			current = true;
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(crud.$$.fragment, local);

    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(crud.$$.fragment, local);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			destroy_component(crud, detaching);
    		}
    	};
    }

    function create_fragment$3(ctx) {
    	var header, div0, a, t1, div1, t2, div2, t3, main, current_block_type_index, if_block1, current;

    	function select_block_type(changed, ctx) {
    		if (ctx.$loggedIn) return create_if_block_3;
    		if (ctx.showRegister) return create_if_block_4;
    		if (ctx.showLogin) return create_if_block_5;
    	}

    	var current_block_type = select_block_type(null, ctx);
    	var if_block0 = current_block_type && current_block_type(ctx);

    	var if_block_creators = [
    		create_if_block$1,
    		create_if_block_1$1,
    		create_if_block_2
    	];

    	var if_blocks = [];

    	function select_block_type_1(changed, ctx) {
    		if (ctx.$loggedIn) return 0;
    		if (ctx.showLogin) return 1;
    		if (ctx.showRegister) return 2;
    		return -1;
    	}

    	if (~(current_block_type_index = select_block_type_1(null, ctx))) {
    		if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
    	}

    	return {
    		c: function create() {
    			header = element("header");
    			div0 = element("div");
    			a = element("a");
    			a.textContent = "actix crud";
    			t1 = space();
    			div1 = element("div");
    			t2 = space();
    			div2 = element("div");
    			if (if_block0) if_block0.c();
    			t3 = space();
    			main = element("main");
    			if (if_block1) if_block1.c();
    			attr(a, "href", "/");
    			add_location(a, file$3, 21, 8, 455);
    			attr(div0, "class", "brand svelte-104w8eq");
    			add_location(div0, file$3, 20, 4, 429);
    			attr(div1, "class", "menu");
    			add_location(div1, file$3, 23, 4, 497);
    			attr(div2, "class", "login");
    			add_location(div2, file$3, 24, 4, 524);
    			attr(header, "class", "svelte-104w8eq");
    			add_location(header, file$3, 19, 0, 416);
    			attr(main, "class", "svelte-104w8eq");
    			add_location(main, file$3, 35, 0, 867);
    		},

    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, header, anchor);
    			append(header, div0);
    			append(div0, a);
    			append(header, t1);
    			append(header, div1);
    			append(header, t2);
    			append(header, div2);
    			if (if_block0) if_block0.m(div2, null);
    			insert(target, t3, anchor);
    			insert(target, main, anchor);
    			if (~current_block_type_index) if_blocks[current_block_type_index].m(main, null);
    			current = true;
    		},

    		p: function update(changed, ctx) {
    			if (current_block_type === (current_block_type = select_block_type(changed, ctx)) && if_block0) {
    				if_block0.p(changed, ctx);
    			} else {
    				if (if_block0) if_block0.d(1);
    				if_block0 = current_block_type && current_block_type(ctx);
    				if (if_block0) {
    					if_block0.c();
    					if_block0.m(div2, null);
    				}
    			}

    			var previous_block_index = current_block_type_index;
    			current_block_type_index = select_block_type_1(changed, ctx);
    			if (current_block_type_index !== previous_block_index) {
    				if (if_block1) {
    					group_outros();
    					transition_out(if_blocks[previous_block_index], 1, 1, () => {
    						if_blocks[previous_block_index] = null;
    					});
    					check_outros();
    				}

    				if (~current_block_type_index) {
    					if_block1 = if_blocks[current_block_type_index];
    					if (!if_block1) {
    						if_block1 = if_blocks[current_block_type_index] = if_block_creators[current_block_type_index](ctx);
    						if_block1.c();
    					}
    					transition_in(if_block1, 1);
    					if_block1.m(main, null);
    				} else {
    					if_block1 = null;
    				}
    			}
    		},

    		i: function intro(local) {
    			if (current) return;
    			transition_in(if_block1);
    			current = true;
    		},

    		o: function outro(local) {
    			transition_out(if_block1);
    			current = false;
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(header);
    			}

    			if (if_block0) if_block0.d();

    			if (detaching) {
    				detach(t3);
    				detach(main);
    			}

    			if (~current_block_type_index) if_blocks[current_block_type_index].d();
    		}
    	};
    }

    function instance$3($$self, $$props, $$invalidate) {
    	let $loggedIn, $username;

    	validate_store(loggedIn, 'loggedIn');
    	component_subscribe($$self, loggedIn, $$value => { $loggedIn = $$value; $$invalidate('$loggedIn', $loggedIn); });
    	validate_store(username, 'username');
    	component_subscribe($$self, username, $$value => { $username = $$value; $$invalidate('$username', $username); });

    	

        let showLogin = !$loggedIn;
        let showRegister = false;

        function showLoginDialog() {
            $$invalidate('showRegister', showRegister = false);
            $$invalidate('showLogin', showLogin = true);
        }

        function showRegisterDialog() {
            $$invalidate('showLogin', showLogin = false);
            $$invalidate('showRegister', showRegister = true);
        }

    	return {
    		showLogin,
    		showRegister,
    		showLoginDialog,
    		showRegisterDialog,
    		$loggedIn,
    		$username
    	};
    }

    class App extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance$3, create_fragment$3, safe_not_equal, []);
    	}
    }

    const app = new App({
        target: document.body,
    });

    return app;

}());
//# sourceMappingURL=bundle.js.map
