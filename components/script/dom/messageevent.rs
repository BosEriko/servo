/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::EventBinding::EventMethods;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding;
use crate::dom::bindings::codegen::Bindings::MessageEventBinding::MessageEventMethods;
use crate::dom::bindings::codegen::UnionTypes::WindowProxyOrMessagePortOrServiceWorker;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::reflector::reflect_dom_object;
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::bindings::utils::message_ports_to_frozen_array;
use crate::dom::event::Event;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::windowproxy::WindowProxy;
use crate::script_runtime::JSContext;
use dom_struct::dom_struct;
use js::jsapi::Heap;
use js::jsval::JSVal;
use js::rust::HandleValue;
use servo_atoms::Atom;

#[unrooted_must_root_lint::must_root]
#[derive(JSTraceable, MallocSizeOf)]
enum SrcObject {
    WindowProxy(Dom<WindowProxy>),
    MessagePort(Dom<MessagePort>),
    ServiceWorker(Dom<ServiceWorker>),
}

impl From<&WindowProxyOrMessagePortOrServiceWorker> for SrcObject {
    #[allow(unrooted_must_root)]
    fn from(src_object: &WindowProxyOrMessagePortOrServiceWorker) -> SrcObject {
        match src_object {
            WindowProxyOrMessagePortOrServiceWorker::WindowProxy(blob) => {
                SrcObject::WindowProxy(Dom::from_ref(&*blob))
            },
            WindowProxyOrMessagePortOrServiceWorker::MessagePort(stream) => {
                SrcObject::MessagePort(Dom::from_ref(&*stream))
            },
            WindowProxyOrMessagePortOrServiceWorker::ServiceWorker(stream) => {
                SrcObject::ServiceWorker(Dom::from_ref(&*stream))
            },
        }
    }
}

#[dom_struct]
pub struct MessageEvent {
    event: Event,
    #[ignore_malloc_size_of = "mozjs"]
    data: Heap<JSVal>,
    origin: DOMString,
    source: Option<SrcObject>,
    lastEventId: DOMString,
    ports: Vec<DomRoot<MessagePort>>,
}

impl MessageEvent {
    pub fn new_uninitialized(global: &GlobalScope) -> DomRoot<MessageEvent> {
        MessageEvent::new_initialized(
            global,
            HandleValue::undefined(),
            DOMString::new(),
            None,
            DOMString::new(),
            vec![],
        )
    }

    pub fn new_initialized(
        global: &GlobalScope,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = Box::new(MessageEvent {
            event: Event::new_inherited(),
            data: Heap::default(),
            source: source.map(|source| source.into()),
            origin,
            lastEventId,
            ports,
        });
        let ev = reflect_dom_object(ev, global, MessageEventBinding::Wrap);
        ev.data.set(data.get());

        ev
    }

    pub fn new(
        global: &GlobalScope,
        type_: Atom,
        bubbles: bool,
        cancelable: bool,
        data: HandleValue,
        origin: DOMString,
        source: Option<&WindowProxyOrMessagePortOrServiceWorker>,
        lastEventId: DOMString,
        ports: Vec<DomRoot<MessagePort>>,
    ) -> DomRoot<MessageEvent> {
        let ev = MessageEvent::new_initialized(global, data, origin, source, lastEventId, ports);
        {
            let event = ev.upcast::<Event>();
            event.init_event(type_, bubbles, cancelable);
        }
        ev
    }

    pub fn Constructor(
        global: &GlobalScope,
        type_: DOMString,
        init: RootedTraceableBox<MessageEventBinding::MessageEventInit>,
    ) -> Fallible<DomRoot<MessageEvent>> {
        let ev = MessageEvent::new(
            global,
            Atom::from(type_),
            init.parent.bubbles,
            init.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.source.as_ref(),
            init.lastEventId.clone(),
            init.ports.clone(),
        );
        Ok(ev)
    }
}

impl MessageEvent {
    pub fn dispatch_jsval(
        target: &EventTarget,
        scope: &GlobalScope,
        message: HandleValue,
        origin: Option<&str>,
        source: Option<&WindowProxy>,
        ports: Vec<DomRoot<MessagePort>>,
    ) {
        let messageevent = MessageEvent::new(
            scope,
            atom!("message"),
            false,
            false,
            message,
            DOMString::from(origin.unwrap_or("")),
            source
                .map(|source| {
                    WindowProxyOrMessagePortOrServiceWorker::WindowProxy(DomRoot::from_ref(source))
                })
                .as_ref(),
            DOMString::new(),
            ports,
        );
        messageevent.upcast::<Event>().fire(target);
    }

    pub fn dispatch_error(target: &EventTarget, scope: &GlobalScope) {
        let init = MessageEventBinding::MessageEventInit::empty();
        let messageevent = MessageEvent::new(
            scope,
            atom!("messageerror"),
            init.parent.bubbles,
            init.parent.cancelable,
            init.data.handle(),
            init.origin.clone(),
            init.source.as_ref(),
            init.lastEventId.clone(),
            init.ports.clone(),
        );
        messageevent.upcast::<Event>().fire(target);
    }
}

impl MessageEventMethods for MessageEvent {
    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-data>
    fn Data(&self, _cx: JSContext) -> JSVal {
        self.data.get()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-origin>
    fn Origin(&self) -> DOMString {
        self.origin.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-messageevent-source
    fn GetSource(&self) -> Option<WindowProxyOrMessagePortOrServiceWorker> {
        match &self.source {
            Some(SrcObject::WindowProxy(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::WindowProxy(DomRoot::from_ref(&*i)),
            ),
            Some(SrcObject::MessagePort(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::MessagePort(DomRoot::from_ref(&*i)),
            ),
            Some(SrcObject::ServiceWorker(i)) => Some(
                WindowProxyOrMessagePortOrServiceWorker::ServiceWorker(DomRoot::from_ref(&*i)),
            ),
            None => None,
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-lasteventid>
    fn LastEventId(&self) -> DOMString {
        self.lastEventId.clone()
    }

    /// <https://dom.spec.whatwg.org/#dom-event-istrusted>
    fn IsTrusted(&self) -> bool {
        self.event.IsTrusted()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageevent-ports>
    fn Ports(&self, cx: JSContext) -> JSVal {
        message_ports_to_frozen_array(self.ports.as_slice(), cx)
    }
}
