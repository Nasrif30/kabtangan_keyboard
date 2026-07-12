import React, { useState, useEffect, useRef } from 'react';
import { Globe, Settings, Clipboard, Delete } from 'lucide-react';

const THEMES = ['light', 'dark', 'forest', 'traditional', 'glass'];

const MOCK_DICTIONARY = [
  'ngaran', 'aku', 'ikaw', 'tausug', 'bahasa', 'sulat', 'marayaw', 'adlaw'
];

export default function App() {
  const [theme, setTheme] = useState('light');
  const [text, setText] = useState('');
  const [keyboardMode, setKeyboardMode] = useState<'tausug' | 'normal'>('tausug');
  const [isShift, setIsShift] = useState(false);
  const [predictions, setPredictions] = useState<string[]>(['ngaran', 'tausug', 'marayaw']);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme);
  }, [theme]);

  useEffect(() => {
    const words = text.split(/\s+/);
    const currentWord = words[words.length - 1].toLowerCase();
    
    if (currentWord) {
      const matches = MOCK_DICTIONARY.filter(w => w.startsWith(currentWord));
      setPredictions(matches.length > 0 ? matches.slice(0, 3) : ['...', '...', '...']);
    } else {
      setPredictions(['aku', 'tausug', 'marayaw']);
    }
  }, [text]);

  const handleKeyPress = (key: string) => {
    if (key === 'SHIFT') {
      setIsShift(!isShift);
      return;
    }

    if (key === '⌫') {
      setText(prev => prev.slice(0, -1));
      return;
    }

    if (key === 'ENTER') {
      setText(prev => prev + '\n');
      return;
    }

    if (key === 'SPACE') {
      setText(prev => prev + ' ');
      return;
    }

    if (key === 'LANG') {
      toggleKeyboard();
      return;
    }

    const char = isShift ? key.toUpperCase() : key;
    setText(prev => prev + char);
    if (isShift) setIsShift(false);
    
    setTimeout(() => textareaRef.current?.focus(), 0);
  };

  const handlePredictionClick = (word: string) => {
    if (word === '...') return;
    const words = text.split(/\s+/);
    words.pop();
    setText(words.join(' ') + (words.length > 0 ? ' ' : '') + word + ' ');
    setTimeout(() => textareaRef.current?.focus(), 0);
  };

  const cycleTheme = () => {
    const nextIdx = (THEMES.indexOf(theme) + 1) % THEMES.length;
    setTheme(THEMES[nextIdx]);
  };

  const toggleKeyboard = () => {
    setKeyboardMode(prev => prev === 'tausug' ? 'normal' : 'tausug');
  };

  const Key = ({ label, display, special, action, space, enter }: any) => {
    const handleDown = (e: React.MouseEvent | React.TouchEvent) => {
      e.preventDefault();
      handleKeyPress(action || label);
    };

    let className = 'key';
    if (special) className += ' key-special';
    if (space) className += ' key-space';
    if (enter) className += ' key-enter';
    if (isShift && action === 'SHIFT') className += ' active';

    const renderLabel = isShift && label.length === 1 ? label.toUpperCase() : (display || label);

    return (
      <button 
        className={className} 
        onMouseDown={handleDown}
        onTouchStart={handleDown}
      >
        {!special && <div className="key-popup">{renderLabel}</div>}
        {renderLabel}
      </button>
    );
  };

  return (
    <div className="app-container">
      <div className="controls">
        <button className="theme-btn" onClick={cycleTheme}>
          Theme: {theme.charAt(0).toUpperCase() + theme.slice(1)}
        </button>
      </div>

      <div className="editor-container">
        <textarea
          ref={textareaRef}
          className="text-editor"
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder={keyboardMode === 'tausug' ? "Type in pure Bahasa Sūg..." : "Type normally..."}
          autoFocus
        />
      </div>

      <div className="keyboard-wrapper">
        <div className="keyboard-container">
          <div className="prediction-bar">
            {predictions.map((p, i) => (
              <div 
                key={i} 
                className={`prediction-word ${i === 1 && p !== '...' ? 'highlight' : ''}`}
                onClick={() => handlePredictionClick(p)}
              >
                {p}
              </div>
            ))}
          </div>

          {keyboardMode === 'tausug' ? (
            <>
              <div className="keyboard-row">
                {['w', 'r', 't', 'y', 'u', 'i', 'p'].map(k => <Key key={k} label={k} />)}
              </div>
              <div className="keyboard-row">
                {['a', 's', 'd', 'g', 'h', 'j', 'k', 'l'].map(k => <Key key={k} label={k} />)}
              </div>
              <div className="keyboard-row">
                <Key label="⇧" action="SHIFT" special />
                {['ch', 'b', 'n', 'm', 'ng', 'ny', "'"].map(k => <Key key={k} label={k} />)}
                <Key display={<Delete size={20} />} action="⌫" special />
              </div>
            </>
          ) : (
            <>
              <div className="keyboard-row">
                {['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'].map(k => <Key key={k} label={k} />)}
              </div>
              <div className="keyboard-row">
                {['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l'].map(k => <Key key={k} label={k} />)}
              </div>
              <div className="keyboard-row">
                <Key label="⇧" action="SHIFT" special />
                {['z', 'x', 'c', 'v', 'b', 'n', 'm'].map(k => <Key key={k} label={k} />)}
                <Key display={<Delete size={20} />} action="⌫" special />
              </div>
            </>
          )}
          
          <div className="keyboard-row">
            <Key display="?123" action="NUM" special />
            <Key display={<Globe size={20} />} action="LANG" special />
            <Key display={<Clipboard size={20} />} action="CLIP" special />
            <Key label={keyboardMode === 'tausug' ? "ūt" : "space"} action="SPACE" space />
            <Key display={<Settings size={20} />} action="SET" special />
            <Key label={keyboardMode === 'tausug' ? "balik" : "return"} action="ENTER" special enter />
          </div>
        </div>
      </div>
    </div>
  );
}
