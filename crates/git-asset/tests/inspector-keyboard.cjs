'use strict';
const {test}=require('node:test'),assert=require('node:assert/strict');
const {catalogueFixture}=require('./ux/catalogue-fixture.cjs');
function key(f,key,shiftKey=false){let prevented=false;f.document.dispatchEvent({type:'keydown',key,shiftKey,preventDefault(){prevented=true;}});return prevented;}

test('work inspector excludes hidden back control and wraps keyboard focus',async()=>{
 const f=catalogueFixture();await f.settle();const opener=f.button('Open Fixture sequence');opener.focus();await f.click(opener);
 assert.equal(f.document.activeElement,f.get('close'));assert.equal(f.get('back-work').hidden,true);
 assert.equal(key(f,'Tab',true),true);const last=f.document.activeElement;assert.notEqual(last,f.get('back-work'));assert.equal(f.get('inspector').contains(last),true);
 assert.equal(key(f,'Tab'),true);assert.equal(f.document.activeElement,f.get('close'));
 assert.equal(key(f,'Escape'),true);assert.equal(f.get('inspector').hidden,true);assert.equal(f.document.activeElement,opener);
 assert.equal(key(f,'Escape'),false);
});

test('nested inspector navigation preserves the original return target',async()=>{
 const f=catalogueFixture();await f.settle();const opener=f.button('Open Fixture sequence');opener.focus();await f.click(opener);
 await f.click('Trace impact of source record');assert.equal(f.get('inspector').hidden,false);
 key(f,'Escape');assert.equal(f.document.activeElement,opener);
});

test('removed opener falls back to visible heading and escaped focus is recaptured',async()=>{
 const f=catalogueFixture();await f.settle();const opener=f.button('Open Fixture sequence');opener.focus();await f.click(opener);
 f.get('search').focus();assert.equal(key(f,'Tab'),true);assert.equal(f.document.activeElement,f.get('close'));
 f.get('assets').replaceChildren();key(f,'Escape');assert.equal(f.document.activeElement,f.get('heading'));
});

test('record inspector focuses close and returns to its actual invoking control',async()=>{
 const f=catalogueFixture();await f.settle();const opener=f.get('search');opener.focus();await f.context.inspect(f.card('source'));
 assert.equal(f.document.activeElement,f.get('close'));f.context.close();assert.equal(f.document.activeElement,opener);
});

test('compact navigation toggles explicitly and closes after choosing a view',async()=>{
 const f=catalogueFixture({objectMode:true});await f.settle();await f.click(f.get('menu'));
 assert.equal(f.get('menu').getAttribute('aria-expanded'),'true');assert.equal(f.get('sidebar').classList.contains('navigation-open'),true);
 await f.context.load();assert.equal(f.get('menu').getAttribute('aria-expanded'),'false');assert.equal(f.get('sidebar').classList.contains('navigation-open'),false);
 await f.click(f.get('menu'));await f.context.openObjectManager('atlas');
 assert.equal(f.get('menu').getAttribute('aria-expanded'),'false');assert.equal(f.get('sidebar').classList.contains('navigation-open'),false);
});
