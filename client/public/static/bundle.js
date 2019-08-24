
(function(l, i, v, e) { v = l.createElement(i); v.async = 1; v.src = '//' + (location.host || 'localhost').split(':')[0] + ':35729/livereload.js?snipver=1'; e = l.getElementsByTagName(i)[0]; e.parentNode.insertBefore(v, e)})(document, 'script');
var app = (function () {
    'use strict';

    function noop() { }
    function assign(tar, src) {
        // @ts-ignore
        for (const k in src)
            tar[k] = src[k];
        return tar;
    }
    function is_promise(value) {
        return value && typeof value === 'object' && typeof value.then === 'function';
    }
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
    function children(element) {
        return Array.from(element.childNodes);
    }

    let current_component;
    function set_current_component(component) {
        current_component = component;
    }
    function get_current_component() {
        if (!current_component)
            throw new Error(`Function called outside component initialization`);
        return current_component;
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

    function handle_promise(promise, info) {
        const token = info.token = {};
        function update(type, index, key, value) {
            if (info.token !== token)
                return;
            info.resolved = key && { [key]: value };
            const child_ctx = assign(assign({}, info.ctx), info.resolved);
            const block = type && (info.current = type)(child_ctx);
            if (info.block) {
                if (info.blocks) {
                    info.blocks.forEach((block, i) => {
                        if (i !== index && block) {
                            group_outros();
                            transition_out(block, 1, 1, () => {
                                info.blocks[i] = null;
                            });
                            check_outros();
                        }
                    });
                }
                else {
                    info.block.d(1);
                }
                block.c();
                transition_in(block, 1);
                block.m(info.mount(), info.anchor);
                flush();
            }
            info.block = block;
            if (info.blocks)
                info.blocks[index] = block;
        }
        if (is_promise(promise)) {
            const current_component = get_current_component();
            promise.then(value => {
                set_current_component(current_component);
                update(info.then, 1, info.value, value);
                set_current_component(null);
            }, error => {
                set_current_component(current_component);
                update(info.catch, 2, info.error, error);
                set_current_component(null);
            });
            // if we previously had a then/catch block, destroy it
            if (info.current !== info.pending) {
                update(info.pending, 0);
                return true;
            }
        }
        else {
            if (info.current !== info.then) {
                update(info.then, 1, info.value, promise);
                return true;
            }
            info.resolved = { [info.value]: promise };
        }
    }

    const globals = (typeof window !== 'undefined' ? window : global);
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
    const { Error: Error_1 } = globals;

    const file = "src/Login.svelte";

    // (25:0) {:catch}
    function create_catch_block(ctx) {
    	var a0, t1, a1, t3, t4, if_block1_anchor, dispose;

    	var if_block0 = (ctx.login) && create_if_block_1();

    	var if_block1 = (ctx.register) && create_if_block();

    	return {
    		c: function create() {
    			a0 = element("a");
    			a0.textContent = "Log in";
    			t1 = space();
    			a1 = element("a");
    			a1.textContent = "Register";
    			t3 = space();
    			if (if_block0) if_block0.c();
    			t4 = space();
    			if (if_block1) if_block1.c();
    			if_block1_anchor = empty();
    			add_location(a0, file, 25, 4, 558);
    			add_location(a1, file, 26, 4, 599);

    			dispose = [
    				listen(a0, "click", ctx.loginDialog),
    				listen(a1, "click", ctx.registerDialog)
    			];
    		},

    		m: function mount(target, anchor) {
    			insert(target, a0, anchor);
    			insert(target, t1, anchor);
    			insert(target, a1, anchor);
    			insert(target, t3, anchor);
    			if (if_block0) if_block0.m(target, anchor);
    			insert(target, t4, anchor);
    			if (if_block1) if_block1.m(target, anchor);
    			insert(target, if_block1_anchor, anchor);
    		},

    		p: function update(changed, ctx) {
    			if (ctx.login) {
    				if (!if_block0) {
    					if_block0 = create_if_block_1();
    					if_block0.c();
    					if_block0.m(t4.parentNode, t4);
    				}
    			} else if (if_block0) {
    				if_block0.d(1);
    				if_block0 = null;
    			}

    			if (ctx.register) {
    				if (!if_block1) {
    					if_block1 = create_if_block();
    					if_block1.c();
    					if_block1.m(if_block1_anchor.parentNode, if_block1_anchor);
    				}
    			} else if (if_block1) {
    				if_block1.d(1);
    				if_block1 = null;
    			}
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(a0);
    				detach(t1);
    				detach(a1);
    				detach(t3);
    			}

    			if (if_block0) if_block0.d(detaching);

    			if (detaching) {
    				detach(t4);
    			}

    			if (if_block1) if_block1.d(detaching);

    			if (detaching) {
    				detach(if_block1_anchor);
    			}

    			run_all(dispose);
    		}
    	};
    }

    // (29:4) {#if login}
    function create_if_block_1(ctx) {
    	var form, label0, t0, input0, t1, label1, t2, input1, t3, button;

    	return {
    		c: function create() {
    			form = element("form");
    			label0 = element("label");
    			t0 = text("Username ");
    			input0 = element("input");
    			t1 = space();
    			label1 = element("label");
    			t2 = text("Password ");
    			input1 = element("input");
    			t3 = space();
    			button = element("button");
    			button.textContent = "Login";
    			attr(input0, "name", "username");
    			add_location(input0, file, 30, 28, 736);
    			add_location(label0, file, 30, 12, 720);
    			attr(input1, "name", "password");
    			attr(input1, "type", "password");
    			add_location(input1, file, 31, 28, 794);
    			add_location(label1, file, 31, 12, 778);
    			attr(button, "type", "submit");
    			add_location(button, file, 32, 12, 850);
    			attr(form, "action", "/user/login");
    			attr(form, "method", "POST");
    			add_location(form, file, 29, 8, 666);
    		},

    		m: function mount(target, anchor) {
    			insert(target, form, anchor);
    			append(form, label0);
    			append(label0, t0);
    			append(label0, input0);
    			append(form, t1);
    			append(form, label1);
    			append(label1, t2);
    			append(label1, input1);
    			append(form, t3);
    			append(form, button);
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(form);
    			}
    		}
    	};
    }

    // (37:4) {#if register}
    function create_if_block(ctx) {
    	var form, label0, t0, input0, t1, label1, t2, input1, t3, button;

    	return {
    		c: function create() {
    			form = element("form");
    			label0 = element("label");
    			t0 = text("Username ");
    			input0 = element("input");
    			t1 = space();
    			label1 = element("label");
    			t2 = text("Password ");
    			input1 = element("input");
    			t3 = space();
    			button = element("button");
    			button.textContent = "Register";
    			attr(input0, "name", "username");
    			add_location(input0, file, 38, 28, 1012);
    			add_location(label0, file, 38, 12, 996);
    			attr(input1, "name", "password");
    			attr(input1, "type", "password");
    			add_location(input1, file, 39, 28, 1070);
    			add_location(label1, file, 39, 12, 1054);
    			attr(button, "type", "submit");
    			add_location(button, file, 40, 12, 1126);
    			attr(form, "action", "/user/register");
    			attr(form, "method", "POST");
    			add_location(form, file, 37, 8, 939);
    		},

    		m: function mount(target, anchor) {
    			insert(target, form, anchor);
    			append(form, label0);
    			append(label0, t0);
    			append(label0, input0);
    			append(form, t1);
    			append(form, label1);
    			append(label1, t2);
    			append(label1, input1);
    			append(form, t3);
    			append(form, button);
    		},

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(form);
    			}
    		}
    	};
    }

    // (23:27)      <span>Logged in as {username}
    function create_then_block(ctx) {
    	var span, t0, t1_value = ctx.username + "", t1, t2, t3, a;

    	return {
    		c: function create() {
    			span = element("span");
    			t0 = text("Logged in as ");
    			t1 = text(t1_value);
    			t2 = text(".");
    			t3 = space();
    			a = element("a");
    			a.textContent = "Logout";
    			add_location(span, file, 23, 4, 473);
    			attr(a, "href", "/user/logout");
    			add_location(a, file, 23, 42, 511);
    		},

    		m: function mount(target, anchor) {
    			insert(target, span, anchor);
    			append(span, t0);
    			append(span, t1);
    			append(span, t2);
    			insert(target, t3, anchor);
    			insert(target, a, anchor);
    		},

    		p: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(span);
    				detach(t3);
    				detach(a);
    			}
    		}
    	};
    }

    // (1:0) <script>     let user = fetch('/user').then(r => {         if (r.status !== 200) {             throw new Error(`Got ${r.statusText}
    function create_pending_block(ctx) {
    	return {
    		c: noop,
    		m: noop,
    		p: noop,
    		d: noop
    	};
    }

    function create_fragment(ctx) {
    	var await_block_anchor, promise;

    	let info = {
    		ctx,
    		current: null,
    		token: null,
    		pending: create_pending_block,
    		then: create_then_block,
    		catch: create_catch_block,
    		value: 'username',
    		error: 'null'
    	};

    	handle_promise(promise = ctx.user, info);

    	return {
    		c: function create() {
    			await_block_anchor = empty();

    			info.block.c();
    		},

    		l: function claim(nodes) {
    			throw new Error_1("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, await_block_anchor, anchor);

    			info.block.m(target, info.anchor = anchor);
    			info.mount = () => await_block_anchor.parentNode;
    			info.anchor = await_block_anchor;
    		},

    		p: function update(changed, new_ctx) {
    			ctx = new_ctx;
    			info.block.p(changed, assign(assign({}, ctx), info.resolved));
    		},

    		i: noop,
    		o: noop,

    		d: function destroy(detaching) {
    			if (detaching) {
    				detach(await_block_anchor);
    			}

    			info.block.d(detaching);
    			info.token = null;
    			info = null;
    		}
    	};
    }

    function instance($$self, $$props, $$invalidate) {
    	let user = fetch('/user').then(r => {
            if (r.status !== 200) {
                throw new Error(`Got ${r.statusText} (${r.status}), body: ${r.body}`);
            }
            return r.text();
        });

        let register = false;
        let login = false;

        function loginDialog() {
            $$invalidate('register', register = false);
            $$invalidate('login', login = true);
        }

        function registerDialog() {
            $$invalidate('login', login = false);
            $$invalidate('register', register = true);
        }

    	return {
    		user,
    		register,
    		login,
    		loginDialog,
    		registerDialog
    	};
    }

    class Login extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, instance, create_fragment, safe_not_equal, []);
    	}
    }

    /* src/App.svelte generated by Svelte v3.9.1 */

    const file$1 = "src/App.svelte";

    function create_fragment$1(ctx) {
    	var header, t, main, h1, current;

    	var login = new Login({ $$inline: true });

    	return {
    		c: function create() {
    			header = element("header");
    			login.$$.fragment.c();
    			t = space();
    			main = element("main");
    			h1 = element("h1");
    			h1.textContent = "Hello!";
    			add_location(header, file$1, 10, 0, 115);
    			attr(h1, "class", "svelte-o015nm");
    			add_location(h1, file$1, 14, 4, 165);
    			add_location(main, file$1, 13, 0, 154);
    		},

    		l: function claim(nodes) {
    			throw new Error("options.hydrate only works if the component was compiled with the `hydratable: true` option");
    		},

    		m: function mount(target, anchor) {
    			insert(target, header, anchor);
    			mount_component(login, header, null);
    			insert(target, t, anchor);
    			insert(target, main, anchor);
    			append(main, h1);
    			current = true;
    		},

    		p: noop,

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
    			if (detaching) {
    				detach(header);
    			}

    			destroy_component(login);

    			if (detaching) {
    				detach(t);
    				detach(main);
    			}
    		}
    	};
    }

    class App extends SvelteComponentDev {
    	constructor(options) {
    		super(options);
    		init(this, options, null, create_fragment$1, safe_not_equal, []);
    	}
    }

    const app = new App({
        target: document.body,
    });

    return app;

}());
//# sourceMappingURL=bundle.js.map
